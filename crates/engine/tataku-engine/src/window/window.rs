use crate::prelude::*;
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
use tokio::sync::mpsc::Sender;
use tataku_graphics::prelude::*;
use tataku_input::prelude::MouseButton;

static WINDOW_PROXY: OnceCell<EventLoopProxy<WindowAction>> = OnceCell::const_new();


lazy_static::lazy_static! {
    pub(super) static ref MONITORS: Arc<RwLock<Vec<String>>> = Arc::default();

    pub static ref RENDER_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref RENDER_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    pub static ref INPUT_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref INPUT_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}


pub struct GameWindow<'window> {
    window: &'window OnceCell<WinitWindow>,
    window_creation_barrier: Arc<std::sync::Barrier>,

    graphics: Box<dyn GraphicsEngine + 'window>,
    pub settings: DisplaySettings,

    window_event_sender: Arc<Sender<WindowEvent>>,
    render_data: Vec<Box<dyn TatakuRenderable>>,

    frametime_timer: TatakuInstant,
    input_timer: TatakuInstant,

    close_pending: bool,
    queued_events: Vec<WindowEvent>,

    // input
    controller_input: gilrs::Gilrs,
    /// what finger ids are currently active
    finger_touches: HashSet<u64>,
    // what finger id started the touch, and where is the floating touch location
    touch_pos: Option<(u64, Vector2)>,

    pub init_graphics: Vec<Box<dyn GraphicsInitializer<'window>>>,
    integration_builders: Vec<TatakuIntegrationBuilder>,

    #[cfg(not(feature = "graphics"))]
    _phantom_data: std::marker::PhantomData<&'window ()>,
}
impl<'window> GameWindow<'window> {
    pub fn new(
        window_event_sender: Sender<WindowEvent>,
        window: &'window OnceCell<WinitWindow>,
        window_creation_barrier: Arc<std::sync::Barrier>,
        settings: &Settings,

        init: WindowInitializers<'window>,
    ) -> Self {
        let now = std::time::Instant::now();

        let s = Self {
            window,
            window_creation_barrier,

            graphics: Box::new(tataku_null_renderer::DummyGraphicsEngine),
            settings: settings.display_settings.clone(),

            window_event_sender: Arc::new(window_event_sender),
            // window_event_receiver,
            render_data: Vec::new(),

            frametime_timer: TatakuInstant::now(),
            input_timer: TatakuInstant::now(),

            close_pending: false,
            queued_events: Vec::new(),

            init_graphics: init.graphics_init,
            integration_builders: init.integrations,
            
            // input
            controller_input: gilrs::Gilrs::new().unwrap(),
            finger_touches: HashSet::new(),
            touch_pos: None,
        };

        debug!("window took {:.2}", now.elapsed().as_secs_f32() * 1000.0);

        s
    }

    pub fn run(mut self, event_loop: winit::event_loop::EventLoop<WindowAction>) {
        WINDOW_PROXY.set(event_loop.create_proxy()).unwrap();
        event_loop.run_app(&mut self).expect("nope");
    }

    pub fn dump_atlas() {
        Self::send_action(WindowAction::DumpAtlas);
    }

    fn send_event(&mut self, event: WindowEvent) {
        // try to send without spawning a task.
        if let Err(tokio::sync::mpsc::error::TrySendError::Full(event)) = self.window_event_sender.try_send(event) {
            // warn!("Game event queue full, event is getting queued: {event:?}");
            self.queued_events.push(event);
        }
    }

    fn update(&mut self) {
        // increment input frametime stuff
        let frametime = (self.input_timer.elapsed_and_reset() * 100.0).floor() as u32;
        INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
        INPUT_COUNT.fetch_add(1, SeqCst);

        // check gamepad events
        while let Some(event) = self.controller_input.next_event() {
            let info = self.controller_input.gamepad(event.id);
            if event.event == gilrs::EventType::Connected { info!("new controller: {}", info.name()) }

            self.send_event(WindowEvent::Input(InputType::RawControllerEvent(event, info.name().into(), info.power_info())));
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
            LoadImage::Image(data, on_done) => on_done(self.graphics.load_texture_rgba(&data, [data.width(), data.height()])),

            LoadImage::Font(font, font_size, on_done) => {
                debug!("Loading font {} with size {font_size}", font.name);
                let font_size = FontSize::new(font_size);
                let mut characters = font.characters.write();

                for (&char, _) in font.font.chars() {
                    // generate glyph data
                    let (metrics, bitmap) = font.font.rasterize(char, font_size.f32());

                    // bitmap is a vec of grayscale pixels
                    // we need to turn that into rgba bytes
                    let data = bitmap
                        .into_iter()
                        .flat_map(|gray| [255,255,255, gray])
                        .collect::<Vec<_>>();

                    let Ok(texture) = self.graphics.load_texture_rgba(&data, [metrics.width as u32, metrics.height as u32]) else { panic!("eve broke fonts") };

                    let char_data = CharData { texture, metrics };
                    characters.insert((font_size.u32(), char), char_data);
                }

                // let the font know the size been loaded
                font.loaded_sizes.write().insert(font_size.u32());

                if let Some(on_done) = on_done {
                    on_done(Ok(()));
                }
            }

            LoadImage::FreeTexture(tex) => {
                self.graphics.free_tex(tex);
            }

            LoadImage::CreateRenderTarget((w, h), on_done, callback) => {
                let rt = self.graphics.create_render_target([w, h], Color::TRANSPARENT, callback);
                on_done(rt.ok_or(TatakuError::from("failed")));
            }
            LoadImage::UpdateRenderTarget(target, on_done, callback) => {
                self.graphics.update_render_target(target, callback);
                on_done(Ok(()));
            }

        }

        trace!("Done loading tex");
    }

    fn render(&mut self) {
        let inner_size = self.window().inner_size();
        if inner_size.width == 0 || inner_size.height == 0 { return }


        let frametime = (self.frametime_timer.elapsed_and_reset() * 100.0).floor() as u32;
        RENDER_FRAMETIME.fetch_max(frametime, SeqCst);
        RENDER_COUNT.fetch_add(1, SeqCst);

        let transform = Matrix::identity();

        self.graphics.begin_render();
        let options = DrawOptions::default();
        self.render_data.iter().for_each(|d| {
            d.draw(&options, transform, &mut *self.graphics);
        });

        self.graphics.end_render();

        // apply
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

        self.send_event(WindowEvent::AvailableMonitors(monitors));
    }

    fn set_fullscreen(&mut self, monitor: FullscreenMonitor) {
        if let FullscreenMonitor::Monitor(name) = monitor {
            if let Some(monitor) = self.window()
                .available_monitors()
                .find(|m| m.name().filter(|n| name == *n).is_some())
            {
                self.window().set_fullscreen(Some(
                    winit::window::Fullscreen::Borderless(Some(monitor))
                ));
                return
            }
        }

        // either its not fullscreen, or the monitor wasnt found, so default to windowed
        let [x, y] = self.settings.window_pos;
        self.window().set_fullscreen(None);
        self.window().set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    fn set_vsync(&mut self, vsync: Vsync) {
        self.graphics.set_vsync(vsync);
    }

    pub fn set_clipboard(content: String) -> TatakuResult {
        use clipboard::{ClipboardProvider, ClipboardContext};
        let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();

        ctx
            .map_err(TatakuError::from_boxed_err)
            .and_then(|mut ctx| ctx
                .set_contents(content)
                .map_err(TatakuError::from_boxed_err)
            )
    }

    fn handle_touch_event(&mut self, touch: Touch) -> Option<WindowEvent> {
        match touch {
            Touch { phase:TouchPhase::Started, location, id, .. } => {
                // info!("+ touch id: {id}");

                let touch_pos = Vector2::new(location.x as f32, location.y as f32);

                self.finger_touches.insert(id);

                // if this is the first touch, set touch pos and send events
                // otherwise, dont send events,
                if self.finger_touches.len() == 1 {
                    self.touch_pos = Some((id, touch_pos));

                    self.send_event(WindowEvent::Input(InputType::MouseMove(Vector2::new(location.x as f32, location.y as f32))));
                    Some(WindowEvent::Input(InputType::MousePress(MouseButton::Left)))
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
                if let Some((start_id, _)) = self.touch_pos {
                    if id == start_id {
                        self.touch_pos = None;

                        return Some(WindowEvent::Input(InputType::MouseRelease(MouseButton::Left)))
                    }
                }

                None
            }

            Touch { phase:TouchPhase::Moved, location, id, .. } => {
                let touch_pos = Vector2::new(location.x as f32, location.y as f32);

                if self.finger_touches.len() > 1 {
                    if let Some((start_id, pos)) = &mut self.touch_pos {
                        if id != *start_id { return None }

                        let delta = touch_pos - *pos;
                        let scroll = Vector2::new(
                            delta.x / 10.0,
                            delta.y / 10.0
                        );
                        *pos = touch_pos;

                        return Some(WindowEvent::Input(InputType::MouseScroll(scroll)))
                    }
                }

                Some(WindowEvent::Input(InputType::MouseMove(touch_pos)))
            }

            _ => None,
        }
    }

}

// static fns
impl GameWindow<'_> {
    fn send_action(event: WindowAction) {
        let Some(proxy) = WINDOW_PROXY.get() else { return };
        let _ = proxy.send_event(event);
    }

    pub fn refresh_monitors() {
        Self::send_action(WindowAction::RefreshMonitors);
    }

    pub fn load_texture_data(data: RgbaImage) -> TatakuResult<TextureReference> {
        trace!("loading tex data");

        let (s, r) = sync_channel(1);
        Self::send_action(WindowAction::LoadImage(Box::new(
            LoadImage::Image(data, Box::new(move |r| s.send(r).nope())))
        ));

        // if this unwrap fails, the receiver was dropped, meaning it was never sent, which means the thread is dead, which means give up
        r.recv().unwrap()
    }

    // // this is called from functions without real access to async, so we have to be dumb here
    // pub fn load_font_data(
    //     font: ActualFont, 
    //     size: f32, 
    //     wait_for_complete: bool
    // ) -> TatakuResult<()> {
    //     // NOTE: this will hang the main thread if this is run there
    //     if wait_for_complete {
    //         let (s, r) = sync_channel(1);
    //         Self::send_event(WindowAction::LoadImage(Box::new(LoadImage::Font(
    //             font, 
    //             size, 
    //             Some(Box::new(move |r| s.send(r).nope()))
    //         ))));

    //         return r.recv().unwrap();
    //     } else {
    //         Self::send_event(WindowAction::LoadImage(
    //             Box::new(LoadImage::Font(font, size, None))
    //         ));
    //     }
    //     Ok(())
    // }


    pub fn create_render_target(
        size: (u32, u32), 
        callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + Sync + 'static
    ) -> TatakuResult<RenderTarget> {
        trace!("create render target");

        let (s, r) = sync_channel(1);
        Self::send_action(WindowAction::LoadImage(Box::new(LoadImage::CreateRenderTarget(
            size, 
            Box::new(move |t| s.send(t).nope()), 
            Box::new(callback)
        ))));

        r.recv().unwrap()
    }

    pub fn update_render_target(
        rt: RenderTarget, 
        callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + Sync + 'static
    ) {
        trace!("update render target");

        let (s, r) = sync_channel(1);
        Self::send_action(WindowAction::LoadImage(Box::new(LoadImage::UpdateRenderTarget(
            rt, 
            Box::new(move |t| s.send(t).nope()), 
            Box::new(callback)
        ))));

        let _ = r.recv().unwrap();
    }


    pub fn free_texture(tex: TextureReference) {
        Self::send_action(WindowAction::LoadImage(Box::new(
            LoadImage::FreeTexture(tex)
        )));
    }
}


#[cfg(feature="graphics")]
impl winit::application::ApplicationHandler<WindowAction> for GameWindow<'_> {
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
            while let Some(graphics_init) = self.init_graphics.pop() {
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
            self.window_creation_barrier.wait(); //.await;
        });

        let mut integrations = Vec::new();
        let window_handle = self.window().window_handle().unwrap();
        for integration in self.integration_builders.take() {
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
        self.send_event(WindowEvent::SizeChanged(self.settings.window_size.into()));
        self.send_event(WindowEvent::IntegrationsLoaded(integrations));
        self.refresh_monitors_inner();
        self.send_event(WindowEvent::VsyncModes(self.graphics.vsync_modes()));
    }


    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if self.window.get().is_none() { return }
        self.update();
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: WindowAction
    ) {
        match event {
            WindowAction::LoadImage(event) => self.run_load_image_event(*event),
            WindowAction::ShowCursor => {
                self.window().set_cursor_visible(true);
            }
            WindowAction::HideCursor => {
                self.window().set_cursor_visible(false);
            }

            WindowAction::RequestAttention => self.window().request_user_attention(Some(winit::window::UserAttentionType::Informational)),

            WindowAction::CloseGame => {
                self.close_pending = true;
                // try send because the game might already be dead at this point
                let _ = self.window_event_sender.try_send(WindowEvent::Closed);
            }

            WindowAction::TakeScreenshot(info) => {
                let sender = self.window_event_sender.clone();

                self.graphics.screenshot(Box::new(move |(data, size)| {
                    let _ = sender.try_send(WindowEvent::ScreenshotComplete(data, size, info));
                }));
            },
            WindowAction::RefreshMonitors => self.refresh_monitors_inner(),

            WindowAction::RenderData(data) => {
                self.render_data = data;
                self.window().request_redraw();
            }

            WindowAction::SettingsUpdated(settings) => {
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

            WindowAction::CopyToClipboard(text) => if let Err(e) = Self::set_clipboard(text.to_string()) {
                error!("error copying to clipboard: {e:?}");
            }

            WindowAction::AddEmitter(emitter) => self.graphics.add_emitter(emitter), 

            WindowAction::DumpAtlas => {
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
                let new_size = Vector2::new(new_size.width as f32, new_size.height as f32);

                if new_size != Vector2::ZERO {
                    self.send_event(WindowEvent::SizeChanged(new_size));
                }

                None
            }

            WinitWindowEvent::CloseRequested => {
                event_loop.exit();
                Some(WindowEvent::Closed)
            }
            WinitWindowEvent::DroppedFile(d) => Some(WindowEvent::FileDrop(d)),
            WinitWindowEvent::HoveredFile(d) => Some(WindowEvent::FileHover(d)),
            WinitWindowEvent::Focused(has_focus) => {
                if has_focus {
                    Some(WindowEvent::GotFocus)
                } else {
                    Some(WindowEvent::LostFocus)
                }
            }

            WinitWindowEvent::KeyboardInput {
                event: e @ winit::event::KeyEvent {
                    state: ElementState::Pressed, ..
                }, ..
            } => Some(WindowEvent::Input(InputType::KeyPress(KeyInput::from_event(e)))),
            WinitWindowEvent::KeyboardInput {
                event: e @  winit::event::KeyEvent {
                    state: ElementState::Released, ..
                }, ..
            } => Some(WindowEvent::Input(InputType::KeyRelease(KeyInput::from_event(e)))),

            WinitWindowEvent::CursorMoved { position, .. } => 
                Some(WindowEvent::Input(InputType::MouseMove(Vector2::new(position.x as f32, position.y as f32)))),

            WinitWindowEvent::MouseWheel { delta, .. } => Some(WindowEvent::Input(InputType::MouseScroll(delta2f32(delta)))),
            WinitWindowEvent::MouseInput { state: ElementState::Pressed, button, .. }  => Some(WindowEvent::Input(InputType::MousePress(button.into()))),
            WinitWindowEvent::MouseInput { state: ElementState::Released, button, .. } => Some(WindowEvent::Input(InputType::MouseRelease(button.into()))),

            WinitWindowEvent::Touch(touch) => self.handle_touch_event(touch),
            // WinitWindowEvent::Occluded(_) => todo!(),

            WinitWindowEvent::RedrawRequested => {
                self.render();
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
    winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(s.x as f64, s.y as f64))
}
#[cfg(feature="graphics")]
fn delta2f32(delta: winit::event::MouseScrollDelta) -> Vector2 {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(x, y) => Vector2::new(x, y),
        winit::event::MouseScrollDelta::PixelDelta(p) => Vector2::new(p.x as f32, p.y as f32),
    }
}



#[async_trait]
pub trait GraphicsInitializer<'window> {
    fn name(&self) -> &'static str;
    async fn init(
        &self,
        window: &'window winit::window::Window,
        settings: DisplaySettings
    ) -> TatakuResult<Box<dyn GraphicsEngine + 'window>>;
}

pub struct WindowInitializers<'a> {
    pub integrations: Vec<TatakuIntegrationBuilder>,
    pub graphics_init: Vec<Box<dyn GraphicsInitializer<'a>>>,
}
