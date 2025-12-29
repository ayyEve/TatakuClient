mod conv;

use tataku_engine::*;
use std::sync::atomic::Ordering;

use image::RgbaImage;
use raw_window_handle::HasWindowHandle;
use winit::{
    event::{
        WindowEvent as WinitWindowEvent,
        StartCause,
        Touch,
        TouchPhase,
        ElementState
    },
    event_loop::{ 
        EventLoop,
        ControlFlow, 
        EventLoopProxy,
        ActiveEventLoop,
    },
};
use tokio::sync::{
    OnceCell,
    mpsc::Sender,
};

use tataku::Vector2;
use input::InputType;
use engine::window::{
    FullscreenMonitor,
    WindowCounters,
    WindowInitializers,
};

pub struct WinitWindow<'window> {
    window: Option<&'window OnceCell<winit::window::Window>>,
    proxy: EventLoopProxy<actions::window::WindowAction>,
    mouse_position_sender: engine::triple_buffer::Input<tataku::Vector2>,

    graphics: Box<dyn graphics::RenderingEngine + 'window>,
    settings: settings::display::DisplaySettings,

    window_event_sender: Arc<Sender<window::Event>>,
    render_data: Vec<Box<dyn graphics::TatakuRenderable>>,

    frametime_timer: tataku::Instant,
    input_timer: tataku::Instant,

    close_pending: bool,
    queued_events: Vec<window::Event>,

    // input
    controller_input: input::gilrs::Gilrs,
    /// what finger ids are currently active
    finger_touches: HashSet<u64>,
    // what finger id started the touch, and where is the floating touch location
    touch_pos: Option<(u64, tataku::Vector2)>,

    init: WindowInitializers<'window>,
    counters: WindowCounters,
}
impl<'window> WinitWindow<'window> {
    pub fn new(
        event_loop: &EventLoop<actions::window::WindowAction>,
        values: window::WindowCreateValues<'window, '_>,
    ) -> Self {
        let now = std::time::Instant::now();

        let controller_mappings = values.settings.sdl_controller_mappings.join("\n");
        let controller_input = input::gilrs::GilrsBuilder::new()
            .add_mappings(&controller_mappings)
            .build()
            .unwrap();

        let s = Self {
            window: None,
            proxy: event_loop.create_proxy(),
            counters: values.counters,

            graphics: Box::new(tataku_null_renderer::DummyGraphicsEngine),
            settings: values.settings.display_settings.clone(),

            window_event_sender: Arc::new(values.event_sender),
            mouse_position_sender: values.mouse_position_sender,
            render_data: Vec::new(),

            frametime_timer: tataku::Instant::now(),
            input_timer: tataku::Instant::now(),

            close_pending: false,
            queued_events: Vec::new(),

            init: values.init,
            
            // input
            controller_input,
            finger_touches: HashSet::new(),
            touch_pos: None,
        };

        debug!("window took {:.2}", now.elapsed().as_secs_f32() * 1000.0);

        s
    }

    fn send_event(&mut self, event: window::Event) {
        // try to send without spawning a task.
        if let Err(tokio::sync::mpsc::error::TrySendError::Full(event)) = self.window_event_sender.try_send(event) {
            // warn!("Game event queue full, event is getting queued: {event:?}");
            self.queued_events.push(event);
        }
    }

    fn update(&mut self) {
        // increment input frametime stuff
        let frametime = (self.input_timer.elapsed_and_reset() * 100.0).floor() as u32;
        self.counters.input_frametime.fetch_max(frametime, Ordering::Release);
        self.counters.input_count.fetch_add(1, Ordering::Release);

        // check gamepad events
        while let Some(event) = self.controller_input.next_event() {
            let info = self.controller_input.gamepad(event.id);
            if event.event == input::gilrs::EventType::Connected { 
                info!("new controller: {}", info.name());
            }

            self.send_event(window::Event::Input(input::InputType::RawControllerEvent(
                event, 
                info.name().into(), 
                info.power_info())
            ));
        }

        // send as many queued requests as we can
        loop {
            let Some(event) = self.queued_events.pop() else { break };
            if let Err(tokio::sync::mpsc::error::TrySendError::Full(event)) = self.window_event_sender.try_send(event) {
                // queue is full again (or still full). re-insert this event back at the top of the queue
                self.queued_events.insert(0, event);
                break;
            }
        }

    }

    fn render(&mut self) {
        let inner_size = self.window().inner_size();
        if inner_size.width == 0 || inner_size.height == 0 { return }

        let frametime = (self.frametime_timer.elapsed_and_reset() * 100.0).floor() as u32;
        self.counters.render_frametime.fetch_max(frametime, Ordering::Release);
        self.counters.render_count.fetch_add(1, Ordering::Release);

        let transform = tataku::Matrix::identity();
        let options = graphics::DrawOptions::default();

        self.graphics.begin_render();
        self.graphics.with_renderer(&|graphics| {
            self.render_data.iter().for_each(|d| {
                d.draw(&options, transform, graphics);
            });
        });
        self.graphics.end_render();

        // apply
        // self.window().pre_present_notify(); FIXME: this forces vsync on wayland which is stupid
        let _ = self.graphics.present();

        // update
        self.graphics.update_emitters();
    }

    fn window(&self) -> &'window winit::window::Window {
        self.window.unwrap().get().unwrap()
    }
}

// input and state stuff
impl WinitWindow<'_> {
    fn refresh_monitors_inner(&mut self) {
        let monitors = self.window()
            .available_monitors()
            .filter_map(|m| m.name())
            .collect::<Vec<_>>();

        self.send_event(window::Event::AvailableMonitors(monitors));
    }

    fn set_fullscreen(&mut self, monitor: FullscreenMonitor) {
        if let FullscreenMonitor::Monitor(name) = monitor
        && let Some(monitor) = self.window()
            .available_monitors()
            .find(|m| m.name().filter(|n| name == *n).is_some())
        {
            self.window().set_fullscreen(Some(
                winit::window::Fullscreen::Borderless(Some(monitor))
            ));
            return
        }

        // either its not fullscreen, or the monitor wasnt found, so default to windowed
        let [x, y] = self.settings.window_pos;
        self.window().set_fullscreen(None);
        self.window().set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    fn set_vsync(&mut self, vsync: tataku::Vsync) {
        self.graphics.set_vsync(vsync);
    }

    fn handle_touch_event(&mut self, touch: Touch) -> Option<window::Event> {
        match touch {
            Touch { phase:TouchPhase::Started, location, id, .. } => {
                // info!("+ touch id: {id}");

                let touch_pos = Vector2::new(location.x as f32, location.y as f32);

                self.finger_touches.insert(id);

                // if this is the first touch, set touch pos and send events
                // otherwise, dont send events,
                if self.finger_touches.len() == 1 {
                    self.touch_pos = Some((id, touch_pos));

                    self.send_event(window::Event::Input(InputType::MouseMove(
                        tataku::Vector2::new(location.x as f32, location.y as f32)
                    )));
                    Some(window::Event::Input(InputType::MousePress(
                        input::MouseButton::Left
                    )))
                } else {
                    None
                }
            }

            Touch { phase:TouchPhase::Ended, id, .. } => {
                // info!("- touch id: {id}");

                // remove this id from touches
                self.finger_touches.remove(&id);

                // check for release of first touch.
                // if this was the first touch, set the touch pos to none, and send a click release event
                if let Some((start_id, _)) = self.touch_pos
                && id == start_id {
                    self.touch_pos = None;

                    return Some(window::Event::Input(InputType::MouseRelease(
                        input::MouseButton::Left
                    )))
                }

                None
            }

            Touch { phase: TouchPhase::Moved, location, id, .. } => {
                let touch_pos = Vector2::new(location.x as f32, location.y as f32);

                if self.finger_touches.len() > 1
                && let Some((start_id, pos)) = &mut self.touch_pos {
                    if id != *start_id { return None }

                    let delta = touch_pos - *pos;
                    *pos = touch_pos;

                    return Some(window::Event::Input(InputType::MouseScroll(
                        input::ScrollInput {
                            value: input::ScrollType::Pixels(delta),
                            sensitivity: self.settings.scroll_sensitivity,
                        }
                    )));
                }

                Some(window::Event::Input(InputType::MouseMove(touch_pos)))
            }

            _ => None,
        }
    }

}

impl winit::application::ApplicationHandler<actions::window::WindowAction> for WinitWindow<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.window {
            None => panic!("Window ready to be initialized before self.window is set!"),
            Some(i) if i.initialized() => return,
            _ => {}
        }

        event_loop.set_control_flow(ControlFlow::Poll);

        #[allow(unused_mut)]
        let mut attribs = winit::window::WindowAttributes::default()
            .with_title("Tataku!")
            .with_min_inner_size(conv::to_size(Vector2::ONE))
            .with_inner_size(conv::to_size(self.settings.window_size.into()))
            .with_decorations(!self.settings.hide_decorations)
            ;

        #[cfg(target_os="linux")] {
            use winit::platform::wayland::WindowAttributesExtWayland;

            let name = "tataku-client";
            attribs = WindowAttributesExtWayland::with_name(attribs, name, name);
        }


        let window = event_loop
            .create_window(attribs)
            .expect("Unable to create window");
        window.set_cursor_visible(false);


        // set window icon
        match image::open("resources/icon-small.png") {
            Ok(image) => {
                let width = image.width();
                let height = image.height();

                match winit::window::Icon::from_rgba(image.to_rgba8().into_vec(), width, height) {
                    Ok(icon) => {
                        window.set_window_icon(Some(icon.clone()));

                        #[cfg(target_os="windows")] {
                            use winit::platform::windows::WindowExtWindows;
                            window.set_taskbar_icon(Some(icon));
                        }
                    }
                    Err(e) => warn!("error setting window icon: {e}")
                }
            }
            Err(e) => warn!("error setting window icon: {e}")
        }

        self.window.unwrap().set(window).unwrap();

        info!("Window created");

        // initialize graphics
        let window_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        window_runtime.block_on(async {
            while let Some(graphics_init) = self.init.graphics_init.pop() {
                let window = self.window();
                match graphics_init.init(window, self.settings.clone()).await {
                    Ok(a) => {
                        self.graphics = a;
                        break
                    }
                    Err(e) => {
                        warn!("error initializing {} graphics: {e:?}", graphics_init.name());
                    }
                }
            }

            debug!("done graphics");

            // let the game side know the window is good to go
            self.init.window_creation_barrier.wait(); //.await;
        });

        let mut integrations = Vec::new();
        let window_handle = self.window().window_handle().unwrap();
        for integration in self.init.integrations.take() {
            let Ok(mut i) = (integration.build)() else { continue };
            if let Err(e) = i.init(window_handle) {
                error!("failed to initialize {}: {e:?}", integration.name);
                continue;
            }
            integrations.push(i);
        }

        self.window().set_min_inner_size(Some(conv::to_size(self.settings.window_size.into())));
        self.set_fullscreen(self.settings.fullscreen_monitor.clone());
        self.set_vsync(self.settings.vsync);
        self.send_event(window::Event::SizeChanged(self.settings.window_size.into()));
        self.send_event(window::Event::IntegrationsLoaded(integrations));
        self.refresh_monitors_inner();
        self.send_event(window::Event::VsyncModes(self.graphics.vsync_modes()));
    }


    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if self.window.is_none() { return }
        self.update();
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: actions::window::WindowAction
    ) {
        use actions::window::WindowAction as Action;

        match event {
            Action::LoadTexture(
                data, 
                on_done
            ) => on_done(self.graphics.load_texture_rgba(
                &data, 
                [data.width(), data.height()]
            )),
            Action::FreeTexture(tex) => self.graphics.free_tex(tex),

            Action::ShowCursor => self.window().set_cursor_visible(true),
            Action::HideCursor => self.window().set_cursor_visible(false),

            Action::RequestAttention => self.window().request_user_attention(Some(winit::window::UserAttentionType::Informational)),
            Action::RefreshMonitors => self.refresh_monitors_inner(),

            Action::CloseGame => {
                self.close_pending = true;
                // try send because the game might already be dead at this point
                let _ = self.window_event_sender.try_send(window::Event::Closed);
            }

            Action::TakeScreenshot(info) => {
                let sender = self.window_event_sender.clone();

                self.graphics.screenshot(Box::new(move |(data, size)| {
                    let _ = sender.try_send(window::Event::ScreenshotComplete(data, size, info));
                }));
            }
            
            Action::RenderData(data) => {
                self.render_data = data;
                self.window().request_redraw();
            }

            Action::SettingsUpdated(settings) => {
                if self.settings.fullscreen_monitor != settings.fullscreen_monitor {
                    self.set_fullscreen(settings.fullscreen_monitor.clone());
                }
                if self.settings.hide_decorations != settings.hide_decorations {
                    self.window().set_decorations(!settings.hide_decorations);
                }

                if self.settings.vsync != settings.vsync {
                    self.set_vsync(settings.vsync);
                }
                if self.settings.enable_blur != settings.enable_blur {
                    self.graphics.set_blur(settings.enable_blur);
                }

                self.settings = settings;
            }

            Action::CopyToClipboard(text) => {
                use clipboard::{ ClipboardProvider, ClipboardContext };
                let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();

                if let Err(e) = ctx
                .map_err(tataku::Error::from_boxed_err)
                .and_then(|mut ctx| ctx
                    .set_contents(text.to_string())
                    .map_err(tataku::Error::from_boxed_err)
                ) {
                    error!("error copying to clipboard: {e:?}");
                }
            }

            Action::AddEmitter(emitter) => self.graphics.add_emitter(emitter), 
            Action::DumpAtlas => self.graphics.dump_atlas("/tmp/fuck/"),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WinitWindowEvent,
    ) {
        if self.close_pending { event_loop.exit(); }

        let event = match event {
            WinitWindowEvent::Resized(new_size) => {
                self.graphics.resize([new_size.width, new_size.height]);
                let new_size = Vector2::new(
                    new_size.width as f32, 
                    new_size.height as f32
                );

                if new_size != Vector2::ZERO {
                    self.send_event(window::Event::SizeChanged(new_size));
                }

                None
            }

            WinitWindowEvent::CloseRequested => {
                event_loop.exit();
                Some(window::Event::Closed)
            }
            WinitWindowEvent::DroppedFile(d) => Some(window::Event::FileDrop(d)),
            WinitWindowEvent::HoveredFile(d) => Some(window::Event::FileHover(d)),
            WinitWindowEvent::Focused(has_focus) => {
                if has_focus {
                    Some(window::Event::GotFocus)
                } else {
                    Some(window::Event::LostFocus)
                }
            }

            WinitWindowEvent::KeyboardInput {
                event: e @ winit::event::KeyEvent {
                    state: ElementState::Pressed, ..
                }, ..
            } => Some(window::Event::Input(InputType::KeyPress(
                conv::keyboard::key(&e).unwrap_or_default()
            ))),
            
            WinitWindowEvent::KeyboardInput {
                event: e @  winit::event::KeyEvent {
                    state: ElementState::Released, ..
                }, ..
            } => Some(window::Event::Input(InputType::KeyRelease(
                conv::keyboard::key(&e).unwrap_or_default()
            ))),

            WinitWindowEvent::CursorMoved { position, .. } => {
                let pos = Vector2::new(position.x as f32, position.y as f32);
                self.mouse_position_sender.write(pos);
                None
                //     Some(window::Event::Input(InputType::MouseMove(pos))),
            }

            WinitWindowEvent::MouseWheel { 
                delta, 
                .. 
            } => Some(window::Event::Input(InputType::MouseScroll(
                input::ScrollInput {
                    value: conv::mouse::scroll(delta),
                    sensitivity: self.settings.scroll_sensitivity,
                }
            ))),

            WinitWindowEvent::MouseInput { 
                state: ElementState::Pressed, 
                button, 
                .. 
            }  => Some(window::Event::Input(InputType::MousePress(
                conv::mouse::button(button)
            ))),
            WinitWindowEvent::MouseInput { 
                state: ElementState::Released, 
                button, 
                .. 
            } => Some(window::Event::Input(InputType::MouseRelease(
                conv::mouse::button(button)
            ))),

            WinitWindowEvent::Touch(touch) => self.handle_touch_event(touch),
            // WinitWindowEvent::Occluded(_) => todo!(),

            WinitWindowEvent::RedrawRequested => {
                self.render();
                self.window().request_redraw();
                None
            }

            _ => None
        };

        if let Some(event) = event { self.send_event(event); }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        warn!("window closing");
    }
}



impl<'w> engine::window::Window<'w> for WinitWindow<'w> {
    fn get_texture_manager(&self) -> Box<dyn window::TextureManager> {
        Box::new(WinitTextureManager {
            proxy: self.proxy.clone(),
        })
    }
    fn get_action_sender(&self) -> Box<dyn window::WindowActionSender> {
        Box::new(WinitActionSender(self.proxy.clone()))
    }

    fn run(mut self: Box<Self>, data: &'w mut dyn std::any::Any) {
        let w = data.downcast_mut::<WinitWindowData>().unwrap();
        self.window = Some(&w.winit_window);

        let event_loop = *w.event_loop.take().unwrap();
        event_loop.run_app(&mut self).expect("nope");
    }
}

struct WinitWindowData {
    event_loop: Option<Box<EventLoop<actions::window::WindowAction>>>,
    winit_window: OnceCell<winit::window::Window>,
}


pub const WINIT_CREATOR: window::WindowCreator = window::WindowCreator {
    create,
};

fn create<'w, 's>(
    values: window::WindowCreateValues<'w, 's>,
) -> (Box<dyn window::Window<'w> + 'w>, Box<dyn std::any::Any>) {
    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    
    let window: WinitWindow<'w> = WinitWindow::new(
        &event_loop,
        values
    );
    
    let data = WinitWindowData {
        event_loop: Some(Box::new(event_loop)),
        winit_window: OnceCell::new(),
    };

    (Box::new(window), Box::new(data))
}


struct WinitActionSender(EventLoopProxy<actions::window::WindowAction>);
impl engine::window::WindowActionSender for WinitActionSender {
    fn send(&mut self, action: actions::window::WindowAction) {
        let _ = self.0.send_event(action);
    }
}


struct WinitTextureManager {
    proxy: EventLoopProxy<actions::window::WindowAction>
}
impl engine::window::TextureManager for WinitTextureManager {
    fn dump_atlas(&mut self) {
        let _ = self.proxy.send_event(actions::window::WindowAction::DumpAtlas);
    }
    fn free_texture(&mut self, tex: tataku::TextureReference) {
        let _ = self.proxy.send_event(
            actions::window::WindowAction::FreeTexture(tex)
        );
    }
    fn load_texture_data(&mut self, data: RgbaImage) -> tataku::Result<tataku::TextureReference> {
        trace!("loading tex data");

        let (
            send, 
            receive
        ) = std::sync::mpsc::sync_channel(1);

        let _ = self.proxy.send_event(
            actions::window::WindowAction::LoadTexture(
                data, 
                Box::new(move |a| { let _ = send.send(a); })
            )
        );

        receive.recv().unwrap()
    }
}
