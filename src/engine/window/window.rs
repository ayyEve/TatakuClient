use crate::prelude::*;
use image::RgbaImage;
#[cfg(feature="graphics")]
use winit::{
    event::*,
    event_loop::ControlFlow,
    window::Window as WinitWindow,
};
use souvlaki::{ MediaControls, PlatformConfig };
use tokio::sync::mpsc::{ unbounded_channel, Sender };

static WINDOW_PROXY: OnceCell<winit::event_loop::EventLoopProxy<Game2WindowEvent>> = OnceCell::const_new();
static MEDIA_CONTROLS:OnceCell<Arc<Mutex<MediaControls>>> = OnceCell::const_new();


lazy_static::lazy_static! {
    pub(super) static ref MONITORS: Arc<RwLock<Vec<String>>> = Default::default();

    pub static ref RENDER_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref RENDER_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    pub static ref INPUT_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref INPUT_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}


pub struct GameWindow<'window> {
    #[cfg(feature="graphics")]
    window: &'window std::cell::OnceCell<WinitWindow>,
    window_creation_barrier: Arc<tokio::sync::Barrier>,

    runtime: Rc<tokio::runtime::Runtime>,

    
    #[cfg(feature="graphics")]
    graphics: Box<dyn GraphicsEngine + 'window>,
    settings: DisplaySettings,
    integration_settings: IntegrationSettings,

    game_event_sender: Arc<Sender<Window2GameEvent>>,
    render_data: Vec<Arc<dyn TatakuRenderable>>,

    frametime_timer: Instant,
    input_timer: Instant,

    close_pending: bool,
    queued_events: Vec<Window2GameEvent>,

    // input
    mouse_helper: MouseInputHelper,
    controller_input: gilrs::Gilrs,
    /// what finger ids are currently active
    finger_touches: HashSet<u64>,
    // what finger id started the touch, and where is the floating touch location
    touch_pos: Option<(u64, Vector2)>,
}
#[cfg(feature="graphics")]
impl<'window> GameWindow<'window> {

    pub async fn new(
        game_event_sender: Sender<Window2GameEvent>,
        window: &'window std::cell::OnceCell<WinitWindow>,
        runtime: Rc<tokio::runtime::Runtime>,
        window_creation_barrier: Arc<tokio::sync::Barrier>,
        settings: &Settings,
    ) -> Self {
        let now = std::time::Instant::now();
    
        let s = Self {
            window,
            window_creation_barrier,
            runtime,

            graphics: Box::new(DummyGraphicsEngine),
            settings: settings.display_settings.clone(),
            integration_settings: settings.integrations.clone(),
            
            game_event_sender: Arc::new(game_event_sender),
            // window_event_receiver,
            render_data: Vec::new(),
            
            frametime_timer: Instant::now(),
            input_timer: Instant::now(),
            
            close_pending: false,
            queued_events: Vec::new(),
            
            // input
            mouse_helper: MouseInputHelper::default(),
            controller_input: gilrs::Gilrs::new().unwrap(),
            finger_touches: HashSet::new(),
            touch_pos: None,
        };

        debug!("window took {:.2}", now.elapsed().as_secs_f32() * 1000.0);

        s
    }

    pub fn run(mut self, event_loop: winit::event_loop::EventLoop<Game2WindowEvent>) {
        WINDOW_PROXY.set(event_loop.create_proxy()).unwrap();
        GlobalValueManager::update(Arc::new(WindowSize(self.settings.window_size.into())));

        event_loop.run_app(&mut self).expect("nope");
    }
    
    fn send_game_event(&mut self, event: Window2GameEvent) {
        // try to send without spawning a task.
        if let Err(tokio::sync::mpsc::error::TrySendError::Full(event)) = self.game_event_sender.try_send(event) {

            // // if this is a mouse pos event, clear all previous mouse pos events since we only care about the final mouse position
            // if let GameEvent::WindowEvent(Window2GameEvent::MouseMove(_)) = &event {
            //     self.queued_events.retain(|e|e)
            // }

            warn!("Game event queue full, event is getting queued: {event:?}");
            self.queued_events.push(event);
            // // if the receiver is full, we spawn the sender off and wait for it to be sent
            // let game_event_sender = self.game_event_sender.clone();
            // tokio::spawn(async move { let _ = game_event_sender.send(event).await; });
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
            match event.event {
                gilrs::EventType::Connected => info!("new controller: {}", info.name()),
                _ => {}
            }
            
            self.send_game_event(Window2GameEvent::ControllerEvent(event, Arc::new(info.name().to_owned()), info.power_info()));
        }
        
        // send as many queued requests as we can
        loop {
            let Some(event) = self.queued_events.pop() else { break };
            if let Err(tokio::sync::mpsc::error::TrySendError::Full(event)) = self.game_event_sender.try_send(event) {
                // queue is full again (or still full). re-insert this event back at the top of the queue
                self.queued_events.insert(0, event);
                break;
            }
        }

    }
    
    fn run_load_image_event(&mut self, event: LoadImage) {
        match event {
            LoadImage::Image(data, on_done) => on_done.send(self.graphics.load_texture_rgba(&data.to_vec(), [data.width(), data.height()])).expect("poopy"),
            
            LoadImage::Font(font, font_size, on_done) => {
                info!("Loading font {} with size {}", font.name, font_size);
                let font_size = FontSize::new(font_size);
                let mut characters = font.characters.write();

                for (&char, _codepoint) in font.font.chars() {
                    // generate glyph data
                    let (metrics, bitmap) = font.font.rasterize(char, font_size.f32());

                    // bitmap is a vec of grayscale pixels
                    
                    // // we need to turn that into rgba bytes
                    let data = bitmap.into_iter().map(|gray| [255,255,255, gray]).flatten().collect::<Vec<_>>();
                    // let mut data = Vec::with_capacity(bitmap.len() * 4);
                    // bitmap.into_iter().for_each(|gray| {
                    //     data.push(255); // r
                    //     data.push(255); // g
                    //     data.push(255); // b
                    //     data.push(gray); // a
                    // });
                    
                    let Ok(texture) = self.graphics.load_texture_rgba(&data, [metrics.width as u32, metrics.height as u32]) else { panic!("eve broke fonts") };
                    
                    let char_data = CharData { texture, metrics };
                    characters.insert((font_size.u32(), char), char_data);
                }

                // let the font know the size been loaded
                font.loaded_sizes.write().insert(font_size.u32());
                
                if let Some(on_done) = on_done {
                    on_done.send(Ok(())).expect("uh oh");
                }
            }

            LoadImage::FreeTexture(tex) => {
                self.graphics.free_tex(tex);
            }

            LoadImage::CreateRenderTarget((w, h), on_done, callback) => {
                let rt = self.graphics.create_render_target([w, h], Color::TRANSPARENT_WHITE, callback);
                on_done.send(rt.ok_or(TatakuError::String("failed".to_owned()))).ok().expect("uh oh");
            }
            LoadImage::UpdateRenderTarget(target, on_done, callback) => {
                self.graphics.update_render_target(target, callback);
                on_done.send(()).ok().expect("uh oh");
            }

        }

        trace!("Done loading tex")
    }

    pub fn render(&mut self) {
        let inner_size = self.window().inner_size();
        if inner_size.width == 0 || inner_size.height == 0 { return }


        let frametime = (self.frametime_timer.elapsed_and_reset() * 100.0).floor() as u32;
        RENDER_FRAMETIME.fetch_max(frametime, SeqCst);
        RENDER_COUNT.fetch_add(1, SeqCst);

        let transform = Matrix::identity();
        
        self.graphics.begin_render();
        self.render_data.iter().for_each(|d| {
            let scissor = d.get_scissor();
            if let Some(scissor) = scissor {
                self.graphics.push_scissor(scissor);
            }
            
            d.draw(transform, &mut *self.graphics);

            if scissor.is_some() {
                self.graphics.pop_scissor();
            }
        });

        self.graphics.end_render();

        // apply
        // self.window().pre_present_notify();
        let _ = self.graphics.present();


        // update
        self.graphics.update_emitters();
    }


    fn init_media_controls(&self) {
        info!("init media controls");
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            let handle = match self.window.get().unwrap().window_handle() {
                Ok(h) =>  match h.as_raw() {
                    RawWindowHandle::Win32(h) => h,
                    // RawWindowHandle::WinRt(h) => h,
                    _ => unreachable!(),
                }
                Err(e) => {
                    error!("error getting raw window handle: {e:?}");
                    return
                }
            };
            Some(handle.hwnd.get() as *mut std::ffi::c_void)
        };

        let config = PlatformConfig {
            dbus_name: "tataku.player",
            display_name: "Tataku!",
            hwnd,
        };
        
        let controls = MediaControls::new(config).unwrap();
        let _ = MEDIA_CONTROLS.set(Arc::new(Mutex::new(controls)));
    }


    fn window(&self) -> &'window WinitWindow {
        self.window.get().unwrap()
    }
}

// input and state stuff
impl<'window> GameWindow<'window> {
    pub fn get_media_controls() -> Arc<Mutex<MediaControls>> {
        MEDIA_CONTROLS.get().cloned().unwrap()
    }

    #[cfg(feature="graphics")]
    fn refresh_monitors_inner(&mut self) {
        *MONITORS.write() = self.window().available_monitors().filter_map(|m|m.name()).collect();
    }

    #[cfg(feature="graphics")]
    fn set_fullscreen(&mut self, monitor: FullscreenMonitor) {
        if let FullscreenMonitor::Monitor(monitor_num) = monitor {
            if let Some((_, monitor)) = self.window().available_monitors().enumerate().find(|(n, _)|*n == monitor_num) {
                self.window().set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor))));
                return
            }
        }

        // either its not fullscreen, or the monitor wasnt found, so default to windowed
        // self.window.apply_windowed();
        let [x,y] = self.settings.window_pos;
        self.window().set_fullscreen(None);
        self.window().set_outer_position(winit::dpi::PhysicalPosition::new(x, y))
    }

    #[cfg(feature="graphics")]
    fn set_vsync(&mut self, vsync: Vsync) {
        self.graphics.set_vsync(vsync);
    }

    pub fn set_clipboard(content: String) -> TatakuResult {
        use clipboard::{ClipboardProvider, ClipboardContext};
        let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();
        
        Ok(ctx
            .map_err(|e|TatakuError::String(e.to_string()))
            .and_then(|mut ctx| ctx.set_contents(content).map_err(|e|TatakuError::String(e.to_string())))?)
    }


    #[cfg(feature="graphics")]
    fn handle_touch_event(&mut self, touch: Touch) -> Option<Window2GameEvent> {
        match touch {
            Touch { phase:TouchPhase::Started, location, id, .. } => {
                // info!("+ touch id: {id}");

                let touch_pos = Vector2::new(location.x as f32, location.y as f32);

                self.finger_touches.insert(id);

                // if this is the first touch, set touch pos and send events
                // otherwise, dont send events, 
                if self.finger_touches.len() == 1 {
                    self.touch_pos = Some((id, touch_pos));
                    
                    self.send_game_event(Window2GameEvent::MouseMove(Vector2::new(location.x as f32, location.y as f32)));
                    Some(Window2GameEvent::MousePress(MouseButton::Left))
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

                        return Some(Window2GameEvent::MouseRelease(MouseButton::Left))
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
                        let y_scroll = delta.y / 10.0;
                        *pos = touch_pos;
                        
                        return Some(Window2GameEvent::MouseScroll(y_scroll))
                    }
                }

                Some(Window2GameEvent::MouseMove(touch_pos))
            }

            _ => None,
        }
    }

    #[cfg(feature="graphics")]
    fn post_cursor_move(&mut self) {
        // if self.mouse_helper.check_bounds(&self.window) {
        //     let Ok(pos) = self.window.inner_position() else { return };
        //     let size = self.window.inner_size();
        //     let x = pos.x as f64 + size.width as f64 / 2.0;
        //     let y = pos.y as f64 + size.height as f64 / 2.0;

        //     let _ = self.window.set_cursor_position(winit::dpi::LogicalPosition::new(x, y));
        // }
    }

}

// static fns
impl<'window> GameWindow<'window> {
    pub fn send_event(event: Game2WindowEvent) {
        // tokio::sync::mpsc::UnboundedReceiver::poll_recv(&mut self, cx)
        #[cfg(feature="graphics")]
        let Some(proxy) = WINDOW_PROXY.get() else { return };
        let _ = proxy.send_event(event);
        // WINDOW_EVENT_QUEUE.get().unwrap().send(event).ok().unwrap();
    }

    pub fn refresh_monitors() {
        Self::send_event(Game2WindowEvent::RefreshMonitors);
    }
    
    pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<TextureReference> {
        trace!("loading tex data");

        let (sender, mut receiver) = unbounded_channel();
        Self::send_event(Game2WindowEvent::LoadImage(LoadImage::Image(data, sender)));

        // if this unwrap fails, the receiver was dropped, meaning it was never sent, which means the thread is dead, which means give up
        receiver.recv().await.unwrap()
    }

    // this is called from functions without real access to async, so we have to be dumb here
    pub fn load_font_data(font: ActualFont, size:f32, wait_for_complete: bool) -> TatakuResult<()> {
        // NOTE: this will hang the main thread if this is run there
        if wait_for_complete {
            let (sender, mut receiver) = unbounded_channel();
            Self::send_event(Game2WindowEvent::LoadImage(LoadImage::Font(font, size, Some(sender))));

            loop {
                match receiver.try_recv() {
                    Ok(_t) => return Ok(()),
                    Err(_) => {},
                }
            }
        } else {
            Self::send_event(Game2WindowEvent::LoadImage(LoadImage::Font(font, size, None)));
        }
        Ok(())
    }


    pub async fn create_render_target(size: (u32, u32), callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + 'static) -> TatakuResult<RenderTarget> {
        trace!("create render target");

        let (sender, mut receiver) = unbounded_channel();
        Self::send_event(Game2WindowEvent::LoadImage(LoadImage::CreateRenderTarget(size, sender, Box::new(callback))));

        receiver.recv().await.unwrap()
    }

    #[allow(unused)]
    pub async fn update_render_target(rt:RenderTarget, callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + 'static) {
        trace!("update render target");

        let (sender, mut receiver) = unbounded_channel();
        Self::send_event(Game2WindowEvent::LoadImage(LoadImage::UpdateRenderTarget(rt, sender, Box::new(callback))));

        receiver.recv().await;
    }


    pub fn free_texture(tex: TextureReference) {
        Self::send_event(Game2WindowEvent::LoadImage(LoadImage::FreeTexture(tex)));
    }
}


impl<'window> winit::application::ApplicationHandler<Game2WindowEvent> for GameWindow<'window> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.get().is_some() { return }
        event_loop.set_control_flow(ControlFlow::Poll);
        
        let window = event_loop.create_window(
            winit::window::WindowAttributes::default()
            .with_title("Tataku!")
            .with_min_inner_size(to_size(Vector2::ONE))
            .with_inner_size(to_size(self.settings.window_size.into()))
        ).expect("Unable to create window");
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


        self.runtime.clone().block_on(async {
            let graphics = GraphicsState::new(self.window(), &self.settings).await;
            self.graphics = Box::new(graphics);
            debug!("done graphics");
            
            // let the game side know the window is good to go
            self.window_creation_barrier.wait().await;
        });


        self.init_media_controls();
        self.window().set_min_inner_size(Some(to_size(self.settings.window_size.into())));
        self.refresh_monitors_inner();
        self.set_fullscreen(self.settings.fullscreen_monitor);
        self.set_vsync(self.settings.vsync);
    }
    

    fn new_events(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _cause: StartCause) {
        if self.window.get().is_none() { return }
        self.update();
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Game2WindowEvent) {
        match event {
            Game2WindowEvent::LoadImage(event) => self.run_load_image_event(event),
            Game2WindowEvent::ShowCursor => { 
                self.mouse_helper.set_system_cursor(true);
                self.window().set_cursor_visible(true);
            }
            Game2WindowEvent::HideCursor => { 
                self.mouse_helper.set_system_cursor(false);
                self.window().set_cursor_visible(false);
            }

            Game2WindowEvent::RequestAttention => self.window().request_user_attention(Some(winit::window::UserAttentionType::Informational)),

            Game2WindowEvent::CloseGame => { 
                self.close_pending = true;
                // try send because the game might already be dead at this point
                let _ = self.game_event_sender.try_send(Window2GameEvent::Closed);
            }

            Game2WindowEvent::TakeScreenshot(info) => self.graphics.screenshot(Box::new(move |(window_data, [width, height])| { 
                // let _ = fuze.send((window_data, width, height)); 
                todo!()
            })),
            Game2WindowEvent::RefreshMonitors => self.refresh_monitors_inner(),

            Game2WindowEvent::AddEmitter(emitter) => self.graphics.add_emitter(emitter),

            Game2WindowEvent::RenderData(data) => {
                self.render_data = data;
                self.window().request_redraw();
            }

            Game2WindowEvent::IntegrationsChanged(integrations) => {
                if integrations.media_controls && !self.integration_settings.media_controls {
                    MediaControlHelper::set_metadata(&Default::default());
                    MediaControlHelper::set_playback(souvlaki::MediaPlayback::Stopped);
                    let _ = MEDIA_CONTROLS.get().unwrap().lock().detach();
                }
                self.integration_settings = integrations;
            }

            Game2WindowEvent::SettingsUpdated(settings) => {
                if self.settings.fullscreen_monitor != settings.fullscreen_monitor {
                    self.set_fullscreen(settings.fullscreen_monitor);
                }
    
                if self.settings.vsync != settings.vsync {
                    self.set_vsync(settings.vsync);
                }
    
                self.mouse_helper.set_raw_input(settings.raw_mouse_input);

                self.settings = settings;
            }

            Game2WindowEvent::CopyToClipboard(text) => if let Err(e) = Self::set_clipboard(text) {
                error!("error copying to clipboard: {e:?}")
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let event = match event {
            DeviceEvent::MouseMotion { delta: (x, y) } => {
                if let Some(new_pos) = self.mouse_helper.device_mouse_moved((x as f32, y as f32), self.window()) {
                    self.post_cursor_move();
                    Some(Window2GameEvent::MouseMove(new_pos))
                } else {
                    None
                }

            }

            _ => None 
        };

        if let Some(event) = event { self.send_game_event(event); }
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.close_pending { event_loop.exit(); }

        let event =  match event {
            WindowEvent::Resized(new_size) => {
                self.graphics.resize([new_size.width, new_size.height]);
                let new_size = Vector2::new(new_size.width as f32, new_size.height as f32);
                
                if new_size != Vector2::ZERO { 
                    GlobalValueManager::update(Arc::new(WindowSize(new_size)));
                }

                None
            }

            // winit::event::WindowEvent::Moved(_) => todo!(),
            // winit::event::WindowEvent::Destroyed => todo!(),
            WindowEvent::CloseRequested => {
                event_loop.exit();
                Some(Window2GameEvent::Closed)
            }
            WindowEvent::DroppedFile(d) => Some(Window2GameEvent::FileDrop(d)),
            WindowEvent::HoveredFile(d) => Some(Window2GameEvent::FileHover(d)),
            // winit::event::WindowEvent::HoveredFileCancelled => todo!(),
            WindowEvent::Focused(has_focus) => {
                self.mouse_helper.set_focus(has_focus, self.window());
                if has_focus {
                    Some(Window2GameEvent::GotFocus)
                } else {
                    Some(Window2GameEvent::LostFocus)
                }
            }

            // WindowEvent::ReceivedCharacter(c) if !c.is_control() => Some(Window2GameEvent::Char(c)),
            // WindowEvent::KeyboardInput {
            //     event: winit::event::KeyEvent {
            //         logical_key: winit::keyboard::Key::Character(c),
            //         state: ElementState::Pressed, ..
            //     }, ..
            // } if c.len() == 1 => {
            //     Some(Window2GameEvent::Char(c.chars().next().unwrap()))
            // }

            // WindowEvent::KeyboardInput { 
            //     event: winit::event::KeyEvent {
            //         logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home),
            //         state: ElementState::Pressed, ..
            //     }, ..
            // } => {
            //     self.mouse_helper.reset_cursor_pos(self.window());
            //     Some(Window2GameEvent::MouseMove(Vector2::ZERO))
            // }

            WindowEvent::KeyboardInput { 
                event: e @ winit::event::KeyEvent { 
                    state: ElementState::Pressed, .. 
                }, .. 
            } => Some(Window2GameEvent::KeyPress(KeyInput::from_event(e))),
            WindowEvent::KeyboardInput { 
                event: e @  winit::event::KeyEvent { 
                    state: ElementState::Released, .. 
                }, .. 
            } => Some(Window2GameEvent::KeyRelease(KeyInput::from_event(e))),
            
            // winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
            // winit::event::WindowEvent::Ime(_) => todo!(),
            WindowEvent::CursorMoved { position, .. } => if let Some(new_pos) = self.mouse_helper.display_mouse_moved(Vector2::new(position.x as f32, position.y as f32)) {
                self.post_cursor_move();
                Some(Window2GameEvent::MouseMove(new_pos))
            } else {
                None
            }
            // winit::event::WindowEvent::CursorEntered { device_id:_ } => todo!(),
            // winit::event::WindowEvent::CursorLeft { device_id:_ } => { self.mouse_pos = None; return },
            WindowEvent::MouseWheel { delta, .. } => Some(Window2GameEvent::MouseScroll(delta2f32(delta))),
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. }  => Some(Window2GameEvent::MousePress(button)),
            WindowEvent::MouseInput { state: ElementState::Released, button, .. } => Some(Window2GameEvent::MouseRelease(button)),
            // winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!();

            WindowEvent::Touch(touch) => self.handle_touch_event(touch),

            WindowEvent::Occluded(_) => todo!(),

            WindowEvent::RedrawRequested => {
                self.render();
                None
            }
        
            _ => None
        };

        if let Some(event) = event { self.send_game_event(event); }
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
fn delta2f32(delta: winit::event::MouseScrollDelta) -> f32 {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(_, y) => y,
        winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
    }
}

