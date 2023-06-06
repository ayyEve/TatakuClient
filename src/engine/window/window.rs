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

/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;


pub static GAME_EVENT_SENDER: OnceCell<Sender<GameEvent>> = OnceCell::const_new();
pub static WINDOW_EVENT_QUEUE:OnceCell<SyncSender<WindowEvent>> = OnceCell::const_new();
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
    event_loop: winit::event_loop::EventLoopProxy<()>,
    graphics: GraphicsState,

    window_event_receiver: Receiver<WindowEvent>,

    frametime_timer: Instant,
    input_timer: Instant,

    settings: SettingsHelper,
    frametime_logger: FrameTimeLogger,

    close_pending: bool,
    texture_load_receiver: UnboundedReceiver<LoadImage>
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
            // let _ = GRAPHICS.set(graphics);
            let _ = RENDER_EVENT_RECEIVER.set(render_event_receiver);
            let _ = GAME_EVENT_SENDER.set(game_event_sender);

            // gl::Enable(gl::DEBUG_OUTPUT);
            // gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            // extern "system" fn gl_callback(_src:u32, _t:u32, _id:u32, _severity:u32, _len:i32, msg: *const i8, _p: *mut std::ffi::c_void) {
            //     let e = unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string() };
            //     if e.starts_with("Buffer detailed info") { return }
            //     error!("gl: {e}")
            // }
            // gl::DebugMessageCallback(gl_callback, 0u8 as *const std::ffi::c_void);
        }

        let proxy = event_loop.create_proxy();

        let now = Instant::now();
        let s = Self {
            window,
            event_loop: proxy,
            graphics,

            window_event_receiver,
            texture_load_receiver,
            settings,

            frametime_timer: now,
            input_timer: now,

            frametime_logger: FrameTimeLogger::new(),
            close_pending: false
        };
        
        (s, event_loop)
    }

    pub async fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) {
        // fire event so things get moved around correctly
        let settings = get_settings!().clone();
        GlobalValueManager::update(Arc::new(WindowSize(settings.window_size.into())));

        self.settings.update();

        self.window.set_inner_size(to_size(self.settings.window_size.into()));
        // self.window.set_raw_mouse_input(settings.raw_mouse_input);
        // self.window.set_vsync(settings.vsync);

        // self.event_loop.send_event(());

        macro_rules! close_window {
            (self) => {
                // self.close();
                
                self.close_pending = true;
                let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowClosed);
                self.frametime_logger.write();
                return;
            }
        }
        
        self.refresh_monitors_inner();
        self.apply_fullscreen();
        self.apply_vsync();

        event_loop.run(move |event, _b, control_flow| {
            let event = match event {
                Event::WindowEvent { window_id:_, event } => {
                    match event {
                        winit::event::WindowEvent::Resized(new_size)
                        | winit::event::WindowEvent::ScaleFactorChanged { new_inner_size:&mut new_size, .. } => {
                            self.graphics.resize(new_size);
            
                            GameWindowEvent::Resized(Vector2::new(new_size.width as f32, new_size.height as f32))
                        }


                        // winit::event::WindowEvent::Moved(_) => todo!(),
                        winit::event::WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            GameWindowEvent::Closed
                        }
                        // winit::event::WindowEvent::Destroyed => todo!(),
                        winit::event::WindowEvent::DroppedFile(d) => GameWindowEvent::FileDrop(d),
                        winit::event::WindowEvent::HoveredFile(d) => GameWindowEvent::FileHover(d),
                        // winit::event::WindowEvent::HoveredFileCancelled => todo!(),
                        winit::event::WindowEvent::ReceivedCharacter(c) if !c.is_control() => GameWindowEvent::Text(c.to_string()),
                        winit::event::WindowEvent::Focused(true) => GameWindowEvent::GotFocus,
                        winit::event::WindowEvent::Focused(false) => GameWindowEvent::LostFocus,
                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(key), state: ElementState::Pressed, .. }, .. } => GameWindowEvent::KeyPress(key),
                        winit::event::WindowEvent::KeyboardInput { input:KeyboardInput { virtual_keycode: Some(key), state: ElementState::Released, .. }, .. } => GameWindowEvent::KeyRelease(key),
                        // winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
                        // winit::event::WindowEvent::Ime(_) => todo!(),
                        winit::event::WindowEvent::CursorMoved { position, .. } => GameWindowEvent::MouseMove(Vector2::new(position.x as f32, position.y as f32)),
                        // winit::event::WindowEvent::CursorEntered { device_id } => todo!(),
                        // winit::event::WindowEvent::CursorLeft { device_id } => todo!(),
                        winit::event::WindowEvent::MouseWheel { delta, .. } => GameWindowEvent::MouseScroll(delta2f32(delta)),
                        winit::event::WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => GameWindowEvent::MousePress(button),
                        winit::event::WindowEvent::MouseInput { state: ElementState::Released, button, .. } => GameWindowEvent::MouseRelease(button),
                        // winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
                        // winit::event::WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
                        winit::event::WindowEvent::Touch(t) => GameWindowEvent::MouseMove(Vector2::new(t.location.x as f32, t.location.y as f32)),
                        // winit::event::WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => todo!(),
                        // winit::event::WindowEvent::ThemeChanged(_) => todo!(),
                        // winit::event::WindowEvent::Occluded(_) => todo!(),
                    
                        _ => return
                    }
                }
    
                Event::UserEvent(_) => {
                    *control_flow = ControlFlow::Exit;
                    GameWindowEvent::Closed
                }
                
                Event::MainEventsCleared => {
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
                            WindowEvent::ShowCursor => self.window.set_cursor_visible(true),
                            WindowEvent::HideCursor => self.window.set_cursor_visible(false),
                            WindowEvent::SetRawInput(_val) => {} // self.window.set_raw_mouse_input(val),
        
                            WindowEvent::RequestAttention => self.window.request_user_attention(Some(winit::window::UserAttentionType::Informational)),
                            WindowEvent::SetClipboard(text) => {
                                use clipboard::{ClipboardProvider, ClipboardContext};
        
                                let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();
                                match ctx {
                                    Ok(mut ctx) => if let Err(e) = ctx.set_contents(text) {
                                        error!("[Clipboard] Error: {:?}", e);
                                    }
                                    Err(e) => error!("[Clipboard] Error: {:?}", e),
                                }
                            },
        
                            WindowEvent::CloseGame => { close_window!(self); },
                            // WindowEvent::TakeScreenshot(fuze) => self.screenshot(fuze).await,
                            WindowEvent::RefreshMonitors => self.refresh_monitors_inner(),

                            _ => {}
                        }
                    }
        
                    // increment input frametime stuff
                    let frametime = (self.input_timer.duration_and_reset() * 100.0).floor() as u32;
                    INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
                    INPUT_COUNT.fetch_add(1, SeqCst);
        
                    // actually render
                    let now = Instant::now();
                    self.render();
                    self.frametime_logger.add(now.as_millis());

                    if self.close_pending {
                        *control_flow = ControlFlow::Exit;
                    }
                    return
                }
                
                // Event::DeviceEvent { device_id, event } => todo!(),
                _ => return
            };
    
            tokio::spawn(async move {
                let game_event_sender = GAME_EVENT_SENDER.get().unwrap();
                // if let Err(e)
                let _ = game_event_sender.send(GameEvent::WindowEvent(event)).await;
            });
        });


        // loop {
        //     // // poll window events
        //     // while let Some(e) = self.window.poll_event() {
        //     //     if e == GameWindowEvent::Closed { close_window!(self); }
        //     //     // if e.close_args().is_some() {  }

        //     //     // controller input, only works with glfw though
        //     //     if let Some(event) = self.window.check_controller_input(&e) {
        //     //         let _ = GAME_EVENT_SENDER.get().unwrap().try_send(event);
        //     //     }

        //     //     if let GameWindowEvent::FileDrop(d) = &e {
        //     //         let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::DragAndDrop(d.clone()));
        //     //         continue
        //     //     }


        //     //     let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowEvent(e));
        //     // }

        //     // check render-side events
        //     if let Ok(event) = self.window_event_receiver.try_recv() {
        //         match event {
        //             WindowEvent::ShowCursor => self.window.set_cursor_visible(true),
        //             WindowEvent::HideCursor => self.window.set_cursor_visible(false),
        //             WindowEvent::SetRawInput(_val) => {} // self.window.set_raw_mouse_input(val),

        //             WindowEvent::RequestAttention => self.window.request_user_attention(Some(winit::window::UserAttentionType::Informational)),
        //             WindowEvent::SetClipboard(text) => {
        //                 use clipboard::{ClipboardProvider, ClipboardContext};

        //                 let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();
        //                 match ctx {
        //                     Ok(mut ctx) => if let Err(e) = ctx.set_contents(text) {
        //                         error!("[Clipboard] Error: {:?}", e);
        //                     }
        //                     Err(e) => error!("[Clipboard] Error: {:?}", e),
        //                 }
        //             },

        //             WindowEvent::CloseGame => { close_window!(self); },
        //             WindowEvent::TakeScreenshot(fuze) => self.screenshot(fuze).await,
        //             WindowEvent::RefreshMonitors => self.refresh_monitors_inner(),
        //         }
        //     }

        //     // increment input frametime stuff
        //     let frametime = (self.input_timer.duration_and_reset() * 100.0).floor() as u32;
        //     INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
        //     INPUT_COUNT.fetch_add(1, SeqCst);

        //     // actually render
        //     let now = Instant::now();
        //     self.render().await;
        //     self.frametime_logger.add(now.as_millis());
        //     tokio::task::yield_now().await;
        // }
    }
    
    async fn screenshot(&mut self, fuze: Fuze<(Vec<u8>, u32, u32)>) {
        self.graphics.screenshot(move |(window_data, width, height)| {
            // screenshot is upside down
            let mut window_data2 = Vec::new();
            for i in (0..window_data.len()).step_by(3 * width as usize).rev() {
                window_data2.extend(window_data[i..i + 3 * width as usize].iter());
            }

            // send it off
            fuze.ignite((window_data2, width, height));
        }).await;
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
        // self.window.set_vsync(self.settings.vsync);
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
                let font_size = FontSize::new(font_size);
                let mut characters = font.characters.write();

                for (&char, _codepoint) in font.font.chars() {
                    // generate glyph data
                    let (metrics, bitmap) = font.font.rasterize(char, font_size.f32());

                    // bitmap is a vec of grayscale pixels
                    // we need to turn that into rgba bytes
                    // TODO: could reduce ram usage during font rasterization if this is moved to where the tex is actually loaded
                    let mut data = Vec::with_capacity(bitmap.len() * 4);
                    bitmap.into_iter().for_each(|gray| {
                        data.push(255); // r
                        data.push(255); // g
                        data.push(255); // b
                        data.push(gray); // a
                    });
                    
                    // char_data.push((char, data, metrics))
                    let Ok(texture) = self.graphics.load_texture_rgba(&data, metrics.width as u32, metrics.height as u32) else {panic!("eve broke fonts")};
                    
                    let char_data = CharData { texture, metrics };
                    characters.insert((font_size.u32(), char), char_data);
                }

                // if let Ok(datas) = state.load_texture_rgba_many(char_data.iter().map(|(_, data, m)|(data, m.width as u32, m.height as u32)).collect()) {
                //     for (texture, (char, _, metrics)) in datas.into_iter().zip(char_data.into_iter()) {
                //         // setup data

                //         // insert data
                //     }

                //     font.loaded_sizes.write().insert(font_size.u32());
                // } else {
                //     panic!("no atlas space for font")
                // }

                on_done.send(Ok(())).expect("uh oh");
            }

            // LoadImage::CreateRenderTarget((w, h), on_done, callback) => {
            //     // match RenderTarget::new_main_thread(w, h) {
            //     //     Ok(mut render_target) => {
            //     //         let graphics = graphics();
            //     //         render_target.bind();
            //     //         callback(&mut render_target, graphics);
            //     //         render_target.unbind();

            //     //         render_targets.push(render_target.render_target_data.clone());
            //     //         image_data.push(render_target.image.tex.clone());

            //     //         if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") }
            //     //     }
            //     //     Err(e) => {
            //     //         if let Err(_) = on_done.send(Err(e)) { error!("uh oh") }
            //     //     }
            //     // }
            // }

            // LoadImage::UpdateRenderTarget(mut render_target, on_done, callback) => {
            //     // render_target.bind();
            //     // let graphics = graphics();
            //     // callback(&mut render_target, graphics);
            //     // render_target.unbind();

            //     // if let Err(_) = on_done.send(Ok(render_target)) { error!("uh oh") };
            // }
        }

        trace!("Done loading tex");
        
    
    }


    pub fn render(&mut self) {
        unsafe {
            if let Ok(_) = NEW_RENDER_DATA_AVAILABLE.compare_exchange(true, false, Acquire, Relaxed) {
                match RENDER_EVENT_RECEIVER.get_mut().unwrap().read() {
                    TatakuRenderEvent::None => {},
                    TatakuRenderEvent::Draw(data) => {

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
                                // i.draw(graphics, c);
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

                        let _ = self.graphics.render(); //.expect("couldnt draw");
                    }
                }
            }

        }
    }


}

// static fns (mostly helpers)
impl GameWindow {
    pub fn refresh_monitors() {
        let _ = WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::RefreshMonitors);
    }

    
    pub async fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<TextureReference> {
        let path = path.as_ref().to_string_lossy().to_string();
        trace!("loading tex {}", path);

        let (sender, mut receiver) = unbounded_channel();
        get_texture_load_queue().expect("no tex load queue").send(LoadImage::Path(path, sender)).ok().expect("no?");

        if let Some(t) = receiver.recv().await {
            t
        } else {
            Err(TatakuError::String("idk".to_owned()))
        }
    }

    pub async fn load_texture_data(data: RgbaImage) -> TatakuResult<TextureReference> {
        trace!("loading tex data");

        let (sender, mut receiver) = unbounded_channel();
        get_texture_load_queue().expect("no tex load queue").send(LoadImage::Image(data, sender)).ok().expect("no?");

        if let Some(t) = receiver.recv().await {
            t
        } else {
            Err(TatakuError::String("idk".to_owned()))
        }
    }

    pub fn load_font_data(font: Font, size:f32) -> TatakuResult<()> {
        // info!("loading font char ('{ch}',{size})");
        let (sender, mut receiver) = unbounded_channel();
        get_texture_load_queue().expect("no tex load queue").send(LoadImage::Font(font, size, sender)).ok().expect("no?");

        loop {
            match receiver.try_recv() {
                Ok(_t) => {
                    return Ok(())
                },
                Err(_) => {},
            }
        }
    }


    // pub async fn create_render_target(size: (f64, f64), callback: impl FnOnce(&mut RenderTarget, &mut GlGraphics) + Send + 'static) -> TatakuResult<RenderTarget> {
    //     trace!("create render target");

    //     let (sender, mut receiver) = unbounded_channel();
    //     get_texture_load_queue()?.send(LoadImage::CreateRenderTarget(size, sender, Box::new(callback))).ok().expect("no?");

    //     if let Some(t) = receiver.recv().await {
    //         t
    //     } else {
    //         Err(TatakuError::String("idk".to_owned()))
    //     }
    // }

    // pub async fn update_render_target(rt:RenderTarget, callback: impl FnOnce(&mut RenderTarget, &mut GlGraphics) + Send + 'static) -> TatakuResult<RenderTarget> {
    //     trace!("update render target");

    //     let (sender, mut receiver) = unbounded_channel();
    //     get_texture_load_queue()?.send(LoadImage::UpdateRenderTarget(rt, sender, Box::new(callback))).ok().expect("no?");

    //     if let Some(t) = receiver.recv().await {
    //         t
    //     } else {
    //         Err(TatakuError::String("idk".to_owned()))
    //     }
    // }
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
pub enum WindowEvent {
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
    Font(Font, f32, UnboundedSender<TatakuResult<()>>),

    // CreateRenderTarget((f64, f64), UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut RenderTarget, &mut GlGraphics) + Send>),
    // UpdateRenderTarget(RenderTarget, UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut RenderTarget, &mut GlGraphics) + Send>),
}


fn get_texture_load_queue<'a>() -> TatakuResult<&'a UnboundedSender<LoadImage>> {
    TEXTURE_LOAD_QUEUE.get().ok_or(TatakuError::String("texture load queue not set".to_owned()))
}