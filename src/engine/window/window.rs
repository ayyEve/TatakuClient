use crate::prelude::*;
use image::RgbaImage;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoopBuilder, EventLoop},
    window::WindowBuilder
};
use souvlaki::{ MediaControls, PlatformConfig };
use std::sync::atomic::Ordering::{ Acquire, Relaxed };
use tokio::sync::mpsc::{ UnboundedSender, UnboundedReceiver, unbounded_channel, Sender };

static WINDOW_EVENT_QUEUE:OnceCell<UnboundedSender<Game2WindowEvent>> = OnceCell::const_new();
pub static NEW_RENDER_DATA_AVAILABLE:AtomicBool = AtomicBool::new(true);
static MEDIA_CONTROLS:OnceCell<Arc<Mutex<MediaControls>>> = OnceCell::const_new();


lazy_static::lazy_static! {
    pub(super) static ref MONITORS: Arc<RwLock<Vec<String>>> = Default::default();

    pub static ref RENDER_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref RENDER_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    pub static ref INPUT_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref INPUT_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}

pub type RenderData = Vec<Arc<dyn TatakuRenderable>>;

pub struct GameWindow {
    window: winit::window::Window,
    graphics: GraphicsState,
    settings: SettingsHelper,

    game_event_sender: Arc<Sender<Window2GameEvent>>,
    window_event_receiver: UnboundedReceiver<Game2WindowEvent>,
    render_event_receiver: TripleBufferReceiver<RenderData>,

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
impl GameWindow {
    pub async fn new(render_event_receiver: TripleBufferReceiver<RenderData>, game_event_sender: Sender<Window2GameEvent>) -> (Self, EventLoop<()>) {
        let settings = SettingsHelper::new();
        
        let event_loop = EventLoopBuilder::new().build();
        let window = WindowBuilder::new().with_min_inner_size(to_size(Vector2::ONE)).build(&event_loop).expect("Unable to create window");
        let graphics = GraphicsState::new(&window, &settings, window.inner_size().into()).await;
        debug!("done graphics");

        // pre-load fonts
        get_font();
        get_fallback_font();
        get_font_awesome();
        debug!("done fonts");

        
        let (window_event_sender, window_event_receiver) = unbounded_channel(); //sync_channel(30);
        WINDOW_EVENT_QUEUE.set(window_event_sender).ok().expect("bad");
        debug!("done texture load queue");
        
        // init audio
        AudioManager::init_audio().expect("error initializing audio");

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
                    },
                    Err(e) => warn!("error setting window icon: {}", e)
                }
            },
            Err(e) => warn!("error setting window icon: {}", e)
        }

        let s = Self {
            window,
            graphics,
            settings,
            
            game_event_sender: Arc::new(game_event_sender),
            window_event_receiver,
            render_event_receiver,
            
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
        (s, event_loop)
    }

    pub fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) {
        // fire event so things get moved around correctly
        // what??
        let settings = get_settings!().clone();
        GlobalValueManager::update(Arc::new(WindowSize(settings.window_size.into())));

        self.init_media_controls();
        self.settings.update();

        self.window.set_inner_size(to_size(self.settings.window_size.into()));

        self.refresh_monitors_inner();
        self.apply_fullscreen();
        self.apply_vsync();

        event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait_timeout(Duration::from_nanos(5));
            self.update();
            if self.close_pending { *control_flow = ControlFlow::Exit; }

            // control_flow.set_wait_until(instant)

            let event = match event {
                Event::WindowEvent { window_id:_, event } => {
                    match event {
                        winit::event::WindowEvent::Resized(new_size)
                        | winit::event::WindowEvent::ScaleFactorChanged { new_inner_size:&mut new_size, .. } => {
                            self.graphics.resize(new_size);
                            let new_size = Vector2::new(new_size.width as f32, new_size.height as f32);
                            
                            if new_size != Vector2::ZERO { 
                                GlobalValueManager::update(Arc::new(WindowSize(new_size)))
                            };
            
                            return;
                        }


                        // winit::event::WindowEvent::Moved(_) => todo!(),
                        // winit::event::WindowEvent::Destroyed => todo!(),
                        winit::event::WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            Window2GameEvent::Closed
                        }
                        winit::event::WindowEvent::DroppedFile(d) => Window2GameEvent::FileDrop(d),
                        winit::event::WindowEvent::HoveredFile(d) => Window2GameEvent::FileHover(d),
                        // winit::event::WindowEvent::HoveredFileCancelled => todo!(),
                        winit::event::WindowEvent::ReceivedCharacter(c) if !c.is_control() => Window2GameEvent::Char(c),
                        winit::event::WindowEvent::Focused(has_focus) => {
                            self.mouse_helper.set_focus(has_focus, &self.window);
                            if has_focus {
                                Window2GameEvent::GotFocus
                            } else {
                                Window2GameEvent::LostFocus
                            }
                        }

                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Home), state: ElementState::Pressed, .. }, .. } => {
                            self.mouse_helper.reset_cursor_pos(&mut self.window);
                            Window2GameEvent::MouseMove(Vector2::ZERO)
                        }

                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(key), state: ElementState::Pressed, .. }, .. } => Window2GameEvent::KeyPress(key),
                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(key), state: ElementState::Released, .. }, .. } => Window2GameEvent::KeyRelease(key),
                        // winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
                        // winit::event::WindowEvent::Ime(_) => todo!(),
                        winit::event::WindowEvent::CursorMoved { position, .. } => if let Some(new_pos) = self.mouse_helper.display_mouse_moved(Vector2::new(position.x as f32, position.y as f32)) {
                            self.post_cursor_move();
                            Window2GameEvent::MouseMove(new_pos)
                        } else {
                            return
                        }
                        // winit::event::WindowEvent::CursorEntered { device_id:_ } => todo!(),
                        // winit::event::WindowEvent::CursorLeft { device_id:_ } => { self.mouse_pos = None; return },
                        winit::event::WindowEvent::MouseWheel { delta, .. } => Window2GameEvent::MouseScroll(delta2f32(delta)),
                        winit::event::WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => Window2GameEvent::MousePress(button),
                        winit::event::WindowEvent::MouseInput { state: ElementState::Released, button, .. } => Window2GameEvent::MouseRelease(button),
                        // winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!();

                        winit::event::WindowEvent::Touch(touch) => if let Some(event) = self.handle_touch_event(touch) {event} else {return},


                        // winit::event::WindowEvent::Occluded(_) => todo!(),
                    
                        _ => return
                    }
                }
                
                Event::DeviceEvent { device_id:_, event } => {
                    match event {
                        DeviceEvent::MouseMotion { delta: (x, y) } => if let Some(new_pos) = self.mouse_helper.device_mouse_moved((x as f32, y as f32), &self.window) {
                            self.post_cursor_move();
                            Window2GameEvent::MouseMove(new_pos)
                        } else {
                            return
                        }

                        _ => return 
                    }
                }
                
                Event::RedrawRequested(_) => {
                    self.render();
                    return;
                }
                _ => return
            };

            self.send_game_event(event);
        });
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
        let old_fullscreen = self.settings.fullscreen_monitor;
        let old_vsync = self.settings.vsync;
        let old_media_integration = self.settings.integrations.media_controls;

        if self.settings.update() {
            if self.settings.fullscreen_monitor != old_fullscreen {
                self.apply_fullscreen();
            }

            if self.settings.vsync != old_vsync {
                self.apply_vsync();
            }

            self.mouse_helper.set_raw_input(self.settings.raw_mouse_input);

            if old_media_integration != self.settings.integrations.media_controls && !self.settings.integrations.media_controls {
                MediaControlHelper::set_metadata(&Default::default());
                MediaControlHelper::set_playback(souvlaki::MediaPlayback::Stopped);
                let _ = MEDIA_CONTROLS.get().unwrap().lock().detach();
            }
        }

        if let Ok(event) = self.window_event_receiver.try_recv() {
            match event {
                Game2WindowEvent::LoadImage(event) => self.run_load_image_event(event),
                Game2WindowEvent::ShowCursor => { 
                    self.mouse_helper.set_system_cursor(true);
                    self.window.set_cursor_visible(true);
                }
                Game2WindowEvent::HideCursor => { 
                    self.mouse_helper.set_system_cursor(false);
                    self.window.set_cursor_visible(false);
                }

                Game2WindowEvent::RequestAttention => self.window.request_user_attention(Some(winit::window::UserAttentionType::Informational)),

                Game2WindowEvent::CloseGame => { 
                    self.close_pending = true;
                    // try send because the game might already be dead at this point
                    let _ = self.game_event_sender.try_send(Window2GameEvent::Closed);
                }

                Game2WindowEvent::TakeScreenshot(fuze) => self.graphics.screenshot(move |(window_data, width, height)| { fuze.ignite((window_data, width, height)); }),
                Game2WindowEvent::RefreshMonitors => self.refresh_monitors_inner(),
            }
        }

        // increment input frametime stuff
        let frametime = (self.input_timer.duration_and_reset() * 100.0).floor() as u32;
        INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
        INPUT_COUNT.fetch_add(1, SeqCst);

        // request a redraw. the render code is run in the event loop
        self.window.request_redraw();

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
            LoadImage::Image(data, on_done) => on_done.send(self.graphics.load_texture_rgba(&data.to_vec(), data.width(), data.height())).expect("poopy"),
            
            LoadImage::Font(font, font_size, on_done) => {
                info!("Loading font {} with size {}", font.name, font_size);
                let font_size = FontSize::new(font_size);
                let mut characters = font.characters.write();

                for (&char, _codepoint) in font.font.chars() {
                    // generate glyph data
                    let (metrics, bitmap) = font.font.rasterize(char, font_size.f32());

                    // bitmap is a vec of grayscale pixels
                    // we need to turn that into rgba bytes
                    let mut data = Vec::with_capacity(bitmap.len() * 4);
                    bitmap.into_iter().for_each(|gray| {
                        data.push(255); // r
                        data.push(255); // g
                        data.push(255); // b
                        data.push(gray); // a
                    });
                    
                    let Ok(texture) = self.graphics.load_texture_rgba(&data, metrics.width as u32, metrics.height as u32) else { panic!("eve broke fonts") };
                    
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
                let rt = self.graphics.create_render_target(w, h, Color::TRANSPARENT_WHITE, callback);
                on_done.send(rt.ok_or(TatakuError::String("failed".to_owned()))).ok().expect("uh oh");
            }

            // LoadImage::UpdateRenderTarget(target, on_done, callback) => {
            //     self.graphics.update_render_target(target, callback);
            // }
        }

        trace!("Done loading tex")
    }

    pub fn render(&mut self) {
        let inner_size = self.window.inner_size();
        if inner_size.width == 0 || inner_size.height == 0 {
            return
        }

        let Ok(_) = NEW_RENDER_DATA_AVAILABLE.compare_exchange(true, false, Acquire, Relaxed) else { return };
        let data = self.render_event_receiver.read();

        let frametime = (self.frametime_timer.duration_and_reset() * 100.0).floor() as u32;
        RENDER_FRAMETIME.fetch_max(frametime, SeqCst);
        RENDER_COUNT.fetch_add(1, SeqCst);

        let transform = Matrix::identity();
        
        self.graphics.begin();
        data.iter().for_each(|d|d.draw(transform, &mut self.graphics));
        self.graphics.end();

        // apply
        let _ = self.graphics.render_current_surface();
    }


    fn init_media_controls(&self) {
        info!("init media controls");
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
            let handle = match self.window.raw_window_handle() {
                RawWindowHandle::Win32(h) => h,
                _ => unreachable!(),
            };
            Some(handle.hwnd)
        };

        let config = PlatformConfig {
            dbus_name: "tataku.player",
            display_name: "Tataku!",
            hwnd,
        };
        
        let controls = MediaControls::new(config).unwrap();
        let _ = MEDIA_CONTROLS.set(Arc::new(Mutex::new(controls)));
    }
}

// input and state stuff
impl GameWindow {
    pub fn get_media_controls() -> Arc<Mutex<MediaControls>> {
        MEDIA_CONTROLS.get().cloned().unwrap()
    }

    fn refresh_monitors_inner(&mut self) {
        *MONITORS.write() = self.window.available_monitors().filter_map(|m|m.name()).collect();
    }

    fn apply_fullscreen(&mut self) {
        if let FullscreenMonitor::Monitor(monitor_num) = self.settings.fullscreen_monitor {
            if let Some((_, monitor)) = self.window.available_monitors().enumerate().find(|(n, _)|*n == monitor_num) {
                self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor))));
                return
            }
        }

        // either its not fullscreen, or the monitor wasnt found, so default to windowed
        // self.window.apply_windowed();
        let [x,y] = self.settings.window_pos;
        self.window.set_fullscreen(None);
        self.window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y))
    }

    fn apply_vsync(&mut self) {
        self.graphics.set_vsync(self.settings.vsync);
    }

    pub fn set_clipboard(content: String) -> TatakuResult {
        use clipboard::{ClipboardProvider, ClipboardContext};
        let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();
        
        Ok(ctx
        .map_err(|e|TatakuError::String(e.to_string()))
        .and_then(|mut ctx|ctx.set_contents(content).map_err(|e|TatakuError::String(e.to_string())))?)
    }


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
impl GameWindow {
    pub fn send_event(event: Game2WindowEvent) {
        // tokio::sync::mpsc::UnboundedReceiver::poll_recv(&mut self, cx)
        WINDOW_EVENT_QUEUE.get().unwrap().send(event).ok().unwrap();
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
    pub fn load_font_data(font: Font, size:f32, wait_for_complete: bool) -> TatakuResult<()> {
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


    pub async fn create_render_target(size: (u32, u32), callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) -> TatakuResult<RenderTarget> {
        trace!("create render target");

        let (sender, mut receiver) = unbounded_channel();
        Self::send_event(Game2WindowEvent::LoadImage(LoadImage::CreateRenderTarget(size, sender, Box::new(callback))));

        receiver.recv().await.unwrap()
    }

    // pub async fn update_render_target(rt:RenderTarget, callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) -> TatakuResult<RenderTarget> {
    //     trace!("update render target");

    //     let (sender, mut receiver) = unbounded_channel();
    //     get_texture_load_queue()?.send(LoadImage::UpdateRenderTarget(rt, sender, Box::new(callback))).ok().expect("no?");

    //     if let Some(t) = receiver.recv().await {
    //         t
    //     } else {
    //         Err(TatakuError::String("idk".to_owned()))
    //     }
    // }


    pub fn free_texture(tex: TextureReference) {
        Self::send_event(Game2WindowEvent::LoadImage(LoadImage::FreeTexture(tex)));
    }
}



fn to_size(s: Vector2) -> winit::dpi::Size {
    winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(s.x as f64, s.y as f64))
}
fn delta2f32(delta: winit::event::MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(p) => p.y as f32,
    }
}
