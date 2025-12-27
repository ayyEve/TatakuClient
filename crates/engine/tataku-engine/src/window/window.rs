use crate::*;
use image::RgbaImage;
use raw_window_handle::HasWindowHandle;
use winit::{
    window::Window as WinitWindow,
    event::{
        WindowEvent as WinitWindowEvent,
        StartCause,
        Touch,
        TouchPhase,
        ElementState
    },
    event_loop::{ 
        ControlFlow, 
        EventLoopProxy,
        ActiveEventLoop,
    },
};
use tokio::sync::OnceCell;
use tokio::sync::mpsc::Sender;
use std::sync::atomic::Ordering;
use std::sync::mpsc::sync_channel;

use tataku::Vector2;
use input::InputType;
use actions::window::LoadImage;
use engine::window::FullscreenMonitor;

static WINDOW_PROXY: OnceCell<EventLoopProxy<actions::window::WindowAction>> = OnceCell::const_new();

pub struct GameWindow<'window> {
    window: &'window OnceCell<WinitWindow>,
    mouse_position_sender: engine::triple_buffer::Input<tataku::Vector2>,

    graphics: Box<dyn graphics::RenderingEngine + 'window>,
    pub settings: settings::display::DisplaySettings,

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

    #[cfg(not(feature = "graphics"))]
    _phantom_data: std::marker::PhantomData<&'window ()>,
}
impl<'window> GameWindow<'window> {
    pub fn new(
        event_sender: Sender<window::Event>,
        mouse_position_sender: engine::triple_buffer::Input<tataku::Vector2>,
        window: &'window OnceCell<WinitWindow>,
        settings: &settings::Settings,
        
        #[cfg(feature="graphics")] window_counters: WindowCounters,
        init: WindowInitializers<'window>,
    ) -> Self {
        let now = std::time::Instant::now();

        let controller_mappings = settings.sdl_controller_mappings.join("\n");
        let controller_input = input::gilrs::GilrsBuilder::new()
            .add_mappings(&controller_mappings)
            .build()
            .unwrap();

        let s = Self {
            window,
            counters: window_counters,

            graphics: Box::new(tataku_null_renderer::DummyGraphicsEngine),
            settings: settings.display_settings.clone(),

            window_event_sender: Arc::new(event_sender),
            mouse_position_sender,
            // window_event_receiver,
            render_data: Vec::new(),

            frametime_timer: tataku::Instant::now(),
            input_timer: tataku::Instant::now(),

            close_pending: false,
            queued_events: Vec::new(),

            init,
            
            // input
            controller_input,
            finger_touches: HashSet::new(),
            touch_pos: None,
        };

        debug!("window took {:.2}", now.elapsed().as_secs_f32() * 1000.0);

        s
    }

    pub fn run(mut self, event_loop: winit::event_loop::EventLoop<actions::window::WindowAction>) {
        WINDOW_PROXY.set(event_loop.create_proxy()).unwrap();
        event_loop.run_app(&mut self).expect("nope");
    }

    pub fn dump_atlas() {
        Self::send_action(actions::window::WindowAction::DumpAtlas);
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

    fn run_load_image_event(&mut self, event: LoadImage) {
        match event {
            LoadImage::Image(
                data, 
                on_done
            ) => on_done(self.graphics.load_texture_rgba(
                &data, 
                [data.width(), data.height()]
            )),

            LoadImage::FreeTexture(tex) => {
                self.graphics.free_tex(tex);
            }

            _ => {}
        }

        trace!("Done loading tex");
    }

    fn render(&mut self) {
        let inner_size = self.window().inner_size();
        if inner_size.width == 0 || inner_size.height == 0 { return }

        let frametime = (self.frametime_timer.elapsed_and_reset() * 100.0).floor() as u32;
        self.counters.render_frametime.fetch_max(frametime, Ordering::Release);
        self.counters.render_count.fetch_add(1, Ordering::Release);

        let transform = tataku::Matrix::identity();

        self.graphics.begin_render();
        let options = graphics::DrawOptions::default();
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

    fn window(&self) -> &'window WinitWindow {
        self.window.get().unwrap()
    }
}

// input and state stuff
impl GameWindow<'_> {
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

    pub fn set_clipboard(content: String) -> tataku::Result<()> {
        use clipboard::{ ClipboardProvider, ClipboardContext };
        let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();

        ctx
            .map_err(tataku::Error::from_boxed_err)
            .and_then(|mut ctx| ctx
                .set_contents(content)
                .map_err(tataku::Error::from_boxed_err)
            )
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

            Touch { phase:TouchPhase::Moved, location, id, .. } => {
                let touch_pos = Vector2::new(location.x as f32, location.y as f32);

                if self.finger_touches.len() > 1
                && let Some((start_id, pos)) = &mut self.touch_pos {
                    if id != *start_id { return None }

                    let delta = touch_pos - *pos;
                    let scroll = Vector2::new(
                        delta.x / 10.0,
                        delta.y / 10.0
                    );
                    *pos = touch_pos;

                    return Some(window::Event::Input(InputType::MouseScroll { 
                        raw: scroll, 
                        scroll: scroll * self.settings.scroll_sensitivity 
                    }));
                }

                Some(window::Event::Input(InputType::MouseMove(touch_pos)))
            }

            _ => None,
        }
    }

}

// static fns
impl GameWindow<'_> {
    fn send_action(event: actions::window::WindowAction) {
        let Some(proxy) = WINDOW_PROXY.get() else { return };
        let _ = proxy.send_event(event);
    }

    pub fn load_texture_data(data: RgbaImage) -> tataku::Result<tataku::TextureReference> {
        trace!("loading tex data");

        let (s, r) = sync_channel(1);
        Self::send_action(actions::window::WindowAction::LoadImage(Box::new(
            LoadImage::Image(data, Box::new(move |r| s.send(r).nope())))
        ));

        // if this unwrap fails, the receiver was dropped, meaning it was never sent, which means the thread is dead, which means give up
        r.recv().unwrap()
    }

    pub fn free_texture(tex: tataku::TextureReference) {
        Self::send_action(actions::window::WindowAction::LoadImage(Box::new(
            LoadImage::FreeTexture(tex)
        )));
    }
}


#[cfg(feature="graphics")]
impl winit::application::ApplicationHandler<actions::window::WindowAction> for GameWindow<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.get().is_some() { return }
        event_loop.set_control_flow(ControlFlow::Poll);

        #[allow(unused_mut)]
        let mut attribs = winit::window::WindowAttributes::default()
            .with_title("Tataku!")
            .with_min_inner_size(to_size(Vector2::ONE))
            .with_inner_size(to_size(self.settings.window_size.into()))
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

        self.window.set(window).unwrap();
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

        self.window().set_min_inner_size(Some(to_size(self.settings.window_size.into())));
        self.set_fullscreen(self.settings.fullscreen_monitor.clone());
        self.set_vsync(self.settings.vsync);
        self.send_event(window::Event::SizeChanged(self.settings.window_size.into()));
        self.send_event(window::Event::IntegrationsLoaded(integrations));
        self.refresh_monitors_inner();
        self.send_event(window::Event::VsyncModes(self.graphics.vsync_modes()));
    }


    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if self.window.get().is_none() { return }
        self.update();
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: actions::window::WindowAction
    ) {
        use actions::window::WindowAction as Action;

        match event {
            Action::LoadImage(event) => self.run_load_image_event(*event),
            Action::ShowCursor => {
                self.window().set_cursor_visible(true);
            }
            Action::HideCursor => {
                self.window().set_cursor_visible(false);
            }

            Action::RequestAttention => self.window().request_user_attention(Some(winit::window::UserAttentionType::Informational)),

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
            },
            Action::RefreshMonitors => self.refresh_monitors_inner(),

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

            Action::CopyToClipboard(text) => if let Err(e) = Self::set_clipboard(text.to_string()) {
                error!("error copying to clipboard: {e:?}");
            }

            Action::AddEmitter(emitter) => self.graphics.add_emitter(emitter), 

            Action::DumpAtlas => {
                self.graphics.dump_atlas("/tmp/fuck/");
            }
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
            } => Some(window::Event::Input(InputType::KeyPress(input::KeyInput::from_event(e)))),
            WinitWindowEvent::KeyboardInput {
                event: e @  winit::event::KeyEvent {
                    state: ElementState::Released, ..
                }, ..
            } => Some(window::Event::Input(InputType::KeyRelease(input::KeyInput::from_event(e)))),

            WinitWindowEvent::CursorMoved { position, .. } => {
                let pos = Vector2::new(position.x as f32, position.y as f32);
                self.mouse_position_sender.write(pos);
                None
                //     Some(window::Event::Input(InputType::MouseMove(pos))),
            }

            WinitWindowEvent::MouseWheel { 
                delta, 
                .. 
            } => {
                use winit::event::MouseScrollDelta::{ LineDelta, PixelDelta };
                let delta = match delta {
                    LineDelta(x, y) => Vector2::new(x, y),
                    PixelDelta(p) => Vector2::new(p.x as f32, p.y as f32),
                };

                Some(window::Event::Input(InputType::MouseScroll {
                    raw: delta,
                    scroll: delta * self.settings.scroll_sensitivity,
                }))
            }

            WinitWindowEvent::MouseInput { 
                state: ElementState::Pressed, 
                button, 
                .. 
            }  => Some(window::Event::Input(InputType::MousePress(button.into()))),
            WinitWindowEvent::MouseInput { 
                state: ElementState::Released, 
                button, 
                .. 
            } => Some(window::Event::Input(InputType::MouseRelease(button.into()))),

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




#[cfg(feature="graphics")]
fn to_size(s: Vector2) -> winit::dpi::Size {
    winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
        s.x as f64, 
        s.y as f64
    ))
}



#[async_trait]
pub trait GraphicsInitializer<'window> {
    fn name(&self) -> &'static str;
    async fn init(
        &self,
        window: &'window winit::window::Window,
        settings: settings::display::DisplaySettings
    ) -> tataku::Result<Box<dyn graphics::RenderingEngine + 'window>>;
}

pub struct WindowInitializers<'a> {
    pub integrations: Vec<io::TatakuIntegrationBuilder>,
    pub graphics_init: Vec<Box<dyn GraphicsInitializer<'a>>>,
    pub window_creation_barrier: Arc<std::sync::Barrier>,
}


#[cfg(feature="graphics")]
pub struct WindowData {
    pub event_receiver: tokio::sync::mpsc::Receiver<engine::window::Event>,
    pub mouse_position_receiver: triple_buffer::Output<Vector2>,
    pub proxy: winit::event_loop::EventLoopProxy<actions::window::WindowAction>,
}
#[cfg(feature="graphics")]
impl WindowData {
    pub fn send_event(&mut self, action: actions::window::WindowAction) {
        self.proxy.send_event(action).unwrap();
    }
}

#[cfg(feature="graphics")]
#[derive(Clone, Default)]
pub struct WindowCounters {
    pub render_count: Arc<std::sync::atomic::AtomicU32>,
    pub render_frametime: Arc<std::sync::atomic::AtomicU32>,

    pub input_count: Arc<std::sync::atomic::AtomicU32>,
    pub input_frametime: Arc<std::sync::atomic::AtomicU32>,
}
