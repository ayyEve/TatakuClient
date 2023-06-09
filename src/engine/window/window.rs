use crate::prelude::*;
use image::RgbaImage;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoopBuilder, EventLoop},
    window::WindowBuilder,
    platform::windows::EventLoopBuilderExtWindows
};
use std::sync::atomic::Ordering::{ Acquire, Relaxed };
use tokio::sync::mpsc::{ UnboundedSender, UnboundedReceiver, unbounded_channel, Sender };

pub static GAME_EVENT_SENDER: OnceCell<Sender<GameEvent>> = OnceCell::const_new();
pub static WINDOW_EVENT_QUEUE:OnceCell<SyncSender<Game2WindowEvent>> = OnceCell::const_new();
static mut RENDER_EVENT_RECEIVER:OnceCell<TripleBufferReceiver<TatakuRenderEvent>> = OnceCell::const_new();
pub static NEW_RENDER_DATA_AVAILABLE:AtomicBool = AtomicBool::new(true);
pub static TEXTURE_LOAD_QUEUE: OnceCell<UnboundedSender<LoadImage>> = OnceCell::const_new();

lazy_static::lazy_static! {
    pub static ref RENDER_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref RENDER_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    pub static ref INPUT_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref INPUT_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}



pub struct GameWindow {
    window: winit::window::Window,
    graphics: GraphicsState,

    window_event_receiver: Receiver<Game2WindowEvent>,

    frametime_timer: Instant,
    input_timer: Instant,

    settings: SettingsHelper,
    frametime_logger: FrameTimeLogger,

    close_pending: bool,
    texture_load_receiver: UnboundedReceiver<LoadImage>,

    // helper for raw mouse input
    mouse_pos: Option<Vector2>,
}
impl GameWindow {
    pub async fn new(render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>, game_event_sender: Sender<GameEvent>) -> (Self, EventLoop<()>) {
        let settings = SettingsHelper::new();
        
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
        let window: winit::window::Window = WindowBuilder::new().build(&event_loop).expect("Unable to create window");
        
        let (texture_load_sender, texture_load_receiver) = unbounded_channel();
        TEXTURE_LOAD_QUEUE.set(texture_load_sender).ok().expect("bad");
        trace!("texture load queue set");


        let [ww, wh]: [f32; 2] = window.inner_size().into();
        let graphics = GraphicsState::new(&window, &settings, [ww as u32, wh as u32]).await;
        debug!("done graphics");

        // pre-load fonts
        get_font();
        get_fallback_font();
        get_font_awesome();
        debug!("done fonts");

        
        let (window_event_sender, window_event_receiver) = sync_channel(10);
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
        
        unsafe {
            let _ = RENDER_EVENT_RECEIVER.set(render_event_receiver);
            let _ = GAME_EVENT_SENDER.set(game_event_sender);
        }


        let now = Instant::now();
        let s = Self {
            window,
            graphics,

            window_event_receiver,
            texture_load_receiver,
            settings,

            frametime_timer: now,
            input_timer: now,

            frametime_logger: FrameTimeLogger::new(),
            close_pending: false,
            mouse_pos: None,
        };
        
        (s, event_loop)
    }

    pub async fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) {
        // fire event so things get moved around correctly
        // what??
        let settings = get_settings!().clone();
        GlobalValueManager::update(Arc::new(WindowSize(settings.window_size.into())));

        self.settings.update();

        self.window.set_inner_size(to_size(self.settings.window_size.into()));

        self.refresh_monitors_inner();
        self.apply_fullscreen();
        self.apply_vsync();

        event_loop.run(move |event, _, control_flow| {
            self.update();
            if self.close_pending { *control_flow = ControlFlow::Exit; }

            let raw_mouse = self.settings.raw_mouse_input;
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
                        winit::event::WindowEvent::ReceivedCharacter(c) if !c.is_control() => Window2GameEvent::Text(c.to_string()),
                        winit::event::WindowEvent::Focused(true) => Window2GameEvent::GotFocus,
                        winit::event::WindowEvent::Focused(false) => Window2GameEvent::LostFocus,
                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(key), state: ElementState::Pressed, .. }, .. } => Window2GameEvent::KeyPress(key),
                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(key), state: ElementState::Released, .. }, .. } => Window2GameEvent::KeyRelease(key),
                        // winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
                        // winit::event::WindowEvent::Ime(_) => todo!(),
                        winit::event::WindowEvent::CursorMoved { position, .. } if !raw_mouse || (raw_mouse && self.mouse_pos.is_none()) => {
                            let p = Vector2::new(position.x as f32, position.y as f32);
                            self.mouse_pos = Some(p); 
                            // self.window.set_cursor_position(position)

                            // if raw_mouse {error!("setting mouse pos to {p:?}")}
                            Window2GameEvent::MouseMove(p)
                        }
                        // winit::event::WindowEvent::CursorEntered { device_id:_ } => todo!(),
                        winit::event::WindowEvent::CursorLeft { device_id:_ } => { self.mouse_pos = None; return },
                        winit::event::WindowEvent::MouseWheel { delta, .. } => Window2GameEvent::MouseScroll(delta2f32(delta)),
                        winit::event::WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => Window2GameEvent::MousePress(button),
                        winit::event::WindowEvent::MouseInput { state: ElementState::Released, button, .. } => Window2GameEvent::MouseRelease(button),
                        // winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
                        // winit::event::WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
                        winit::event::WindowEvent::Touch(t) => Window2GameEvent::MouseMove(Vector2::new(t.location.x as f32, t.location.y as f32)),
                        // winit::event::WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => todo!(),
                        // winit::event::WindowEvent::Occluded(_) => todo!(),
                    
                        _ => return
                    }
                }
                
                Event::DeviceEvent { device_id:_, event } => {
                    match event {
                        DeviceEvent::MouseMotion { delta: (x, y) } if raw_mouse => {
                            if let Some(mouse_pos) = self.mouse_pos.as_mut() {
                                mouse_pos.x += x as f32;
                                mouse_pos.y += y as f32;
                                Window2GameEvent::MouseMove(*mouse_pos)
                            } else {
                                return
                            }
                        }
                        // DeviceEvent::Added => todo!(),
                        // DeviceEvent::Removed => todo!(),
                        // DeviceEvent::MouseWheel { delta } => todo!(),
                        // DeviceEvent::Motion { axis, value } => todo!(),
                        // DeviceEvent::Button { button, state } => todo!(),
                        // DeviceEvent::Key(_) => todo!(),
                        // DeviceEvent::Text { codepoint } => todo!(),

                        _ => return 
                    }
                }
                
                Event::UserEvent(_) => {
                    *control_flow = ControlFlow::Exit;
                    Window2GameEvent::Closed
                }
                
                Event::RedrawRequested(_) => {
                    self.render();
                    return;
                }
                _ => return
            };
    
            tokio::spawn(async move {
                let game_event_sender = GAME_EVENT_SENDER.get().unwrap();
                // if let Err(e)
                let _ = game_event_sender.send(GameEvent::WindowEvent(event)).await;
            });
        });
    }

    fn update(&mut self) {
        let old_fullscreen = self.settings.fullscreen_monitor;
        let old_vsync = self.settings.vsync;

        if self.settings.update() {
            if self.settings.fullscreen_monitor != old_fullscreen {
                self.apply_fullscreen();
            }

            if self.settings.vsync != old_vsync {
                self.apply_vsync();
            }
        }

        self.check_texture_load_loop();

        if let Ok(event) = self.window_event_receiver.try_recv() {
            match event {
                Game2WindowEvent::ShowCursor => self.window.set_cursor_visible(true),
                Game2WindowEvent::HideCursor => self.window.set_cursor_visible(false),
                Game2WindowEvent::SetRawInput(_val) => {} // self.window.set_raw_mouse_input(val),

                Game2WindowEvent::RequestAttention => self.window.request_user_attention(Some(winit::window::UserAttentionType::Informational)),
                Game2WindowEvent::SetClipboard(text) => {
                    use clipboard::{ClipboardProvider, ClipboardContext};

                    let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();
                    match ctx {
                        Ok(mut ctx) => if let Err(e) = ctx.set_contents(text) {
                            error!("[Clipboard] Error: {:?}", e);
                        }
                        Err(e) => error!("[Clipboard] Error: {:?}", e),
                    }
                },

                Game2WindowEvent::CloseGame => { 
                    self.close_pending = true;
                    let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowClosed);
                    self.frametime_logger.write();
                }

                Game2WindowEvent::TakeScreenshot(fuze) => self.graphics.screenshot(move |(window_data, width, height)| { fuze.ignite((window_data, width, height)); }),
                Game2WindowEvent::RefreshMonitors => self.refresh_monitors_inner(),
            }
        }

        // increment input frametime stuff
        let frametime = (self.input_timer.duration_and_reset() * 100.0).floor() as u32;
        INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
        INPUT_COUNT.fetch_add(1, SeqCst);

        // actually render
        let now = Instant::now();
        self.window.request_redraw();
        self.frametime_logger.add(now.as_millis());
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


    fn check_texture_load_loop(&mut self) {
        let Ok(method) = self.texture_load_receiver.try_recv() else { return };

        match method {
            LoadImage::GameClose => { return; }
            LoadImage::Path(path, on_done) => {
                on_done.send(self.graphics.load_texture_path(&path)).expect("poopy");
            }
            LoadImage::Image(data, on_done) => {
                on_done.send(self.graphics.load_texture_rgba(&data.to_vec(), data.width(), data.height())).expect("poopy");
            }
            LoadImage::Font(font, font_size, on_done) => {
                trace!("Loading font {} with size {}", font.name, font_size);
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
        let data = unsafe {
            let Ok(_) = NEW_RENDER_DATA_AVAILABLE.compare_exchange(true, false, Acquire, Relaxed) else { return };
            let TatakuRenderEvent::Draw(data) = RENDER_EVENT_RECEIVER.get_mut().unwrap().read() else { return };
            data
        };

        let frametime = (self.frametime_timer.duration_and_reset() * 100.0).floor() as u32;
        RENDER_FRAMETIME.fetch_max(frametime, SeqCst);
        RENDER_COUNT.fetch_add(1, SeqCst);

        let transform = Matrix::identity();
        
        // use this for snipping
        #[cfg(feature="snipping")] {
            // let orig_c = graphics.draw_begin(args.viewport());
            self.graphics.begin();

            for i in data.iter() {
                // let mut drawstate_changed = false;
                // let mut c = orig_c;

                // if let Some(ds) = i.get_draw_state() {
                //     drawstate_changed = true;
                //     // println!("ic: {:?}", ic);
                //     graphics.draw_end();
                //     graphics.draw_begin(args.viewport());
                //     graphics.use_draw_state(&ds);
                //     c.draw_state = ds;
                // }
                
                // graphics.use_draw_state(&c.draw_state);
                i.draw(transform, &mut self.graphics);

                // if drawstate_changed {
                //     graphics.draw_end();
                //     graphics.draw_begin(args.viewport());
                //     graphics.use_draw_state(&orig_c.draw_state);
                // }
            }

            self.graphics.end();
        }


        #[cfg(not(feature="snipping"))] {
            let c = graphics.draw_begin(args.viewport());
            graphics::clear(GFX_CLEAR_COLOR.into(), graphics);
            
            for i in data.iter() {
                i.draw(graphics, c);
            }
            
            graphics.draw_end();
        }

        // apply
        let _ = self.graphics.render_current_surface(); //.expect("couldnt draw");
    }

}

// static fns (mostly helpers)
impl GameWindow {
    pub fn refresh_monitors() {
        let _ = WINDOW_EVENT_QUEUE.get().unwrap().send(Game2WindowEvent::RefreshMonitors);
    }

    
    pub async fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<TextureReference> {
        let path = path.as_ref().to_string_lossy().to_string();
        trace!("loading tex {}", path);

        let (sender, mut receiver) = unbounded_channel();
        send_texture_load_event(LoadImage::Path(path, sender));

        receiver.recv().await.unwrap()
    }

    pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<TextureReference> {
        trace!("loading tex data");

        let (sender, mut receiver) = unbounded_channel();
        send_texture_load_event(LoadImage::Image(data, sender));

        // if this unwrap fails, the receiver was dropped, meaning it was never sent, which means the thread is dead, which means give up
        receiver.recv().await.unwrap()
    }

    // this is called from functions without real access to async, so we have to be dumb here
    pub fn load_font_data(font: Font, size:f32, wait_for_complete: bool) -> TatakuResult<()> {
        // NOTE: this will hang the main thread if this is run there
        if wait_for_complete {
            let (sender, mut receiver) = unbounded_channel();
            send_texture_load_event(LoadImage::Font(font, size, Some(sender)));

            loop {
                match receiver.try_recv() {
                    Ok(_t) => return Ok(()),
                    Err(_) => {},
                }
            }
        } else {
            send_texture_load_event(LoadImage::Font(font, size, None));
        }
        Ok(())
    }


    pub async fn create_render_target(size: (u32, u32), callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) -> TatakuResult<RenderTarget> {
        trace!("create render target");

        let (sender, mut receiver) = unbounded_channel();
        send_texture_load_event(LoadImage::CreateRenderTarget(size, sender, Box::new(callback)));

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
        send_texture_load_event(LoadImage::FreeTexture(tex));
    }
}





pub enum TatakuRenderEvent {
    None,
    Draw(Vec<Arc<dyn TatakuRenderable>>),
}
impl Default for TatakuRenderEvent {
    fn default() -> Self {
        Self::None
    }
}



#[allow(unused)]
pub enum Game2WindowEvent {
    ShowCursor,
    HideCursor,
    RequestAttention,
    SetRawInput(bool),
    SetClipboard(String),
    CloseGame,
    TakeScreenshot(Fuze<(Vec<u8>, u32, u32)>),

    RefreshMonitors,
}


lazy_static::lazy_static! {
    static ref MONITORS: Arc<RwLock<Vec<String>>> = Default::default();
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum FullscreenMonitor {
    None,
    Monitor(usize),
}
impl Dropdownable for FullscreenMonitor {
    fn variants() -> Vec<Self> {
        [Self::None].into_iter().chain((0..MONITORS.read().len()).into_iter().map(|t|Self::Monitor(t))).collect()
    }

    fn display_text(&self) -> String {
        match self {
            Self::None => "None".to_owned(),
            Self::Monitor(num) => MONITORS
                .read()
                .get(*num)
                .map(|s|format!("({num}). {s}"))
                .unwrap_or_else(||"None".to_owned())
        }
    }

    fn from_string(s:String) -> Self {
        match s.parse::<usize>() {
            Err(_) => Self::None,
            Ok(num) => Self::Monitor(num)
        }
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




pub enum LoadImage {
    GameClose,
    Path(String, UnboundedSender<TatakuResult<TextureReference>>),
    Image(RgbaImage, UnboundedSender<TatakuResult<TextureReference>>),
    Font(Font, f32, Option<UnboundedSender<TatakuResult<()>>>),
    FreeTexture(TextureReference),

    CreateRenderTarget((u32, u32), UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut GraphicsState, Matrix) + Send>),
    // UpdateRenderTarget(RenderTarget, UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut GraphicsState, Matrix) + Send>),
}


fn send_texture_load_event<'a>(event: LoadImage) {
    TEXTURE_LOAD_QUEUE
    .get()
    .ok_or(TatakuError::String("texture load queue not set".to_owned())).unwrap()
    .send(event)
    .map_err(|_|TatakuError::String("couldnt send event".to_owned())).expect("Couldnt send texture load event");
}