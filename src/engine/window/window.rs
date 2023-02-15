use crate::prelude::*;
use opengl_graphics::GlGraphics;
use std::sync::atomic::Ordering::{ Acquire, Relaxed };
use piston::input::*;

/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;

// pain and suffering
static mut GRAPHICS: OnceCell<GlGraphics> = OnceCell::const_new();
pub static GAME_EVENT_SENDER: OnceCell<tokio::sync::mpsc::Sender<GameEvent>> = OnceCell::const_new();

pub static WINDOW_EVENT_QUEUE:OnceCell<SyncSender<WindowEvent>> = OnceCell::const_new();
static mut RENDER_EVENT_RECEIVER:OnceCell<TripleBufferReceiver<TatakuRenderEvent>> = OnceCell::const_new();
pub static NEW_RENDER_DATA_AVAILABLE:AtomicBool = AtomicBool::new(true);

pub fn graphics() -> &'static mut GlGraphics {
    unsafe {
        GRAPHICS.get_mut().unwrap()
    }
}


lazy_static::lazy_static! {
    pub static ref RENDER_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref RENDER_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));

    pub static ref INPUT_COUNT: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref INPUT_FRAMETIME: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
}



pub struct GameWindow {
    pub window: Box<dyn GameWindowTrait>,

    window_event_receiver: Receiver<WindowEvent>,

    frametime_timer: Instant,
    input_timer: Instant,

    settings: SettingsHelper,

    frametime_logger: FrameTimeLogger,
}

impl GameWindow {
    pub fn start(render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>, gane_event_sender: tokio::sync::mpsc::Sender<GameEvent>) -> Self {
        let window_size = WindowSizeHelper::new();
        let size = [window_size.x as u32, window_size.y as u32];

        let available_windows:Vec<Box<dyn Fn([u32; 2]) -> TatakuResult<Box<dyn GameWindowTrait>>>> = vec![
            #[cfg(feature="glfw_window")]
            Box::new(GlfwGameWindow::create),

            #[cfg(feature="glutin_window")]
            Box::new(GlutinGameWindow::create),
        ];

        let Some(mut window) = available_windows
            .into_iter()
            .find_map(|w|match (w)(size) {
                Ok(w) => Some(w),
                Err(e) => { warn!("error creating window: {e:?}"); None }
            })
        else {
            panic!("unable to create any windows");
        };

        let graphics = GlGraphics::new(window.gl());
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
            Ok(img) => window.set_icon(img.into_rgba8()),
            Err(e) => warn!("error setting window icon: {}", e)
        }
        
        unsafe {
            let _ = GRAPHICS.set(graphics);
            let _ = RENDER_EVENT_RECEIVER.set(render_event_receiver);
            let _ = GAME_EVENT_SENDER.set(gane_event_sender);

            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            extern "system" fn gl_callback(_src:u32, _t:u32, _id:u32, _severity:u32, _len:i32, msg: *const i8, _p: *mut std::ffi::c_void) {
                let e = unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string() };
                if e.starts_with("Buffer detailed info") { return }
                error!("gl: {e}")
            }
            gl::DebugMessageCallback(gl_callback, 0u8 as *const std::ffi::c_void);
        }

        let now = Instant::now();
        Self {
            window,

            window_event_receiver,
            settings: SettingsHelper::new(),

            frametime_timer: now,
            input_timer: now,

            frametime_logger: FrameTimeLogger::new(),
        }
    }

    pub async fn run(&mut self) {
        // fire event so things get moved around correctly
        let settings = get_settings!().clone();
        GlobalValueManager::update(Arc::new(WindowSize(settings.window_size.into())));

        self.settings.update();

        self.window.set_size(self.settings.window_size.into());
        self.window.set_raw_mouse_input(settings.raw_mouse_input);
        self.window.set_vsync(settings.vsync);


        macro_rules! close_window {
            (self) => {
                self.window.close();
                let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowClosed);
                self.frametime_logger.write();
                return;
            }
        }
        
        self.refresh_monitors_inner();
        self.apply_fullscreen();
        self.apply_vsync();

        loop {
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

            // poll window events
            while let Some(e) = self.window.poll_event() {
                if e.close_args().is_some() { close_window!(self); }

                // controller input, only works with glfw though
                if let Some(event) = self.window.check_controller_input(&e) {
                    let _ = GAME_EVENT_SENDER.get().unwrap().try_send(event);
                }

                if let Event::Input(Input::FileDrag(FileDrag::Drop(d)), _) = e {
                    let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::DragAndDrop(d));
                    continue
                }

                let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowEvent(e));
            }

            // check render-side events
            if let Ok(event) = self.window_event_receiver.try_recv() {
                match event {
                    WindowEvent::ShowCursor => self.window.set_cursor_visible(true),
                    WindowEvent::HideCursor => self.window.set_cursor_visible(false),
                    WindowEvent::SetRawInput(val) => self.window.set_raw_mouse_input(val),

                    WindowEvent::RequestAttention => self.window.request_attention(),
                    WindowEvent::SetClipboard(val) => self.window.set_clipboard(val),

                    WindowEvent::CloseGame => { close_window!(self); },
                    WindowEvent::TakeScreenshot(fuze) => self.screenshot(fuze),
                    WindowEvent::RefreshMonitors => self.refresh_monitors_inner(),

                }
            }

            // increment input frametime stuff
            let frametime = (self.input_timer.duration_and_reset() * 100.0).floor() as u32;
            INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
            INPUT_COUNT.fetch_add(1, SeqCst);

            // actually render
            let now = Instant::now();
            self.render().await;
            self.frametime_logger.add(now.as_millis());
            tokio::task::yield_now().await;
        }
    }
    
    async fn render(&mut self) {
        let draw_size = self.window.get_draw_size();

        let args = RenderArgs {
            ext_dt: 0.0,
            window_size: self.window.get_size().into(),
            draw_size: [draw_size.x as u32, draw_size.y as u32],
        };

        render(self.window.get_buffer_swappable(), args, &mut self.frametime_timer);
    }

    fn screenshot(&self, fuze: Fuze<(Vec<u8>, u32, u32)>) {
        let size = self.window.get_size();
        let width  = size.x as usize;
        let height = size.y as usize;

        let data_size = 3 * width * height;
        let mut window_data:Vec<u8> = vec![0; data_size];
        let window_data2 = window_data.as_mut_slice().as_mut_ptr() as *mut std::ffi::c_void;

        unsafe {
            gl::ReadPixels(
                0, 
                0, 
                width as i32, 
                height as i32, 
                gl::RGB, 
                gl::UNSIGNED_BYTE, 
                window_data2
            );
        }

        // screenshot is upside down
        let mut window_data2 = Vec::new();
        for i in (0..window_data.len()).step_by(3 * width).rev() {
            window_data2.extend(window_data[i..i + 3 * width].iter());
        }
        
        // send it off
        fuze.ignite((window_data2, width as u32, height as u32));
    }


    fn refresh_monitors_inner(&mut self) {
        *MONITORS.write() = self.window.get_monitors();
    }

    fn apply_fullscreen(&mut self) {
        if let FullscreenMonitor::Monitor(monitor_num) = self.settings.fullscreen_monitor {
            if self.window.apply_fullscreen(monitor_num) {
                return
            }
        }

        // either its not fullscreen, or the monitor wasnt found, so default to windowed
        self.window.apply_windowed();
    }

    fn apply_vsync(&mut self) {
        self.window.set_vsync(self.settings.vsync);
    }
}

// static fns (mostly helpers)
impl GameWindow {
    pub fn refresh_monitors() {
        let _ = WINDOW_EVENT_QUEUE.get().unwrap().send(WindowEvent::RefreshMonitors);
    }
}

pub fn render<BS: BufferSwappable + ?Sized>(window: &mut BS, args: RenderArgs, frametime: &mut Instant) {
    unsafe {
        if let Ok(_) = NEW_RENDER_DATA_AVAILABLE.compare_exchange(true, false, Acquire, Relaxed) {
            match RENDER_EVENT_RECEIVER.get_mut().unwrap().read() {
                TatakuRenderEvent::None => {},
                TatakuRenderEvent::Draw(data) => {
                    let graphics = graphics();

                    let frametime = (frametime.duration_and_reset() * 100.0).floor() as u32;
                    RENDER_FRAMETIME.fetch_max(frametime, SeqCst);
                    RENDER_COUNT.fetch_add(1, SeqCst);

                    // use this for snipping
                    #[cfg(feature="snipping")] {
                        let orig_c = graphics.draw_begin(args.viewport());
                        graphics::clear(GFX_CLEAR_COLOR.into(), graphics);

                        for i in data.iter() {
                            let mut drawstate_changed = false;
                            let mut c = orig_c;

                            if let Some(ds) = i.get_draw_state() {
                                drawstate_changed = true;
                                // println!("ic: {:?}", ic);
                                graphics.draw_end();
                                graphics.draw_begin(args.viewport());
                                graphics.use_draw_state(&ds);
                                c.draw_state = ds;
                            }
                            
                            // graphics.use_draw_state(&c.draw_state);
                            i.draw(graphics, c);

                            if drawstate_changed {
                                graphics.draw_end();
                                graphics.draw_begin(args.viewport());
                                graphics.use_draw_state(&orig_c.draw_state);
                            }
                        }

                        graphics.draw_end();
                    }

                    #[cfg(not(feature="snipping"))] {
                        let c = graphics.draw_begin(args.viewport());
                        graphics::clear(GFX_CLEAR_COLOR.into(), graphics);
                        
                        for i in data.iter() {
                            i.draw(graphics, c);
                        }
                        
                        graphics.draw_end();
                    }
                    // loop {
                    //     let e = gl::GetError();
                    //     if e == gl::NO_ERROR { break }
                    //     println!("gl error: {e}");
                    // }
                    
                    window.swap_buffers()
                }
            }
        }

    }
}



pub enum TatakuRenderEvent {
    None,
    Draw(Vec<Arc<dyn Renderable>>),
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