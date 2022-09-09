use crate::prelude::*;
use std::sync::atomic::Ordering::{ Acquire, Relaxed };
use glfw::Context;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    input::*, 
    window::WindowSettings,
    Window
};


/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;

// pain and suffering
static mut GRAPHICS: OnceCell<GlGraphics> = OnceCell::const_new();
static mut GAME_EVENT_SENDER: OnceCell<MultiFuze<GameEvent>> = OnceCell::const_new();

pub static WINDOW_EVENT_QUEUE: OnceCell<SyncSender<WindowEvent>> = OnceCell::const_new();
static mut RENDER_EVENT_RECEIVER:OnceCell<TripleBufferReceiver<TatakuRenderEvent>> = OnceCell::const_new();
pub static NEW_RENDER_DATA_AVAILABLE: AtomicBool = AtomicBool::new(true);

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
    pub window: glfw_window::GlfwWindow,
    // pub graphics: GlGraphics,

    // game_event_sender: MultiFuze<GameEvent>,
    // render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>,
    window_event_receiver: Receiver<WindowEvent>,

    #[cfg(feature="bass_audio")]
    #[allow(dead_code)]
    /// needed to prevent bass from deinitializing
    bass: bass_rs::Bass,

    frametime_timer: Instant,
    input_timer: Instant,

    window_size: WindowSizeHelper,
}

impl GameWindow {
    pub fn start(render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>, gane_event_sender: MultiFuze<GameEvent>) -> Self {
        let opengl = OpenGL::V4_5;
        let mut window: glfw_window::GlfwWindow = WindowSettings::new("Tataku!", [20, 20])
            .graphics_api(opengl)
            // .resizable(false)
            // .fullscreen(true) // this doesnt work?
            // .samples(32) // not sure if this actually works or not
            .build()
            .expect("Error creating window");
        // window.window.set_cursor_mode(glfw::CursorMode::Hidden);


        let graphics = GlGraphics::new(opengl);
        info!("done graphics");


        // pre-load fonts
        get_font();
        get_fallback_font();
        get_font_awesome();
        info!("done fonts");

        
        let (window_event_sender, window_event_receiver) = sync_channel(10);
        WINDOW_EVENT_QUEUE.set(window_event_sender).ok().expect("bad");
        info!("done texture load queue");
        
        
        #[cfg(feature="bass_audio")] 
        let bass = {
            #[cfg(target_os = "windows")]
            let window_ptr = window.window.get_win32_window();
            #[cfg(target_os = "linux")]
            let window_ptr = window.window.get_x11_window();
            #[cfg(target_os = "macos")]
            let window_ptr = window.window.get_cocoa_window();

            // initialize bass
            bass_rs::Bass::init_default_with_ptr(window_ptr).expect("Error initializing bass")
        };

        // set window icon
        match image::open("resources/icon-small.png") {
            Ok(img) => window.window.set_icon(vec![img.into_rgba8()]),
            Err(e) => warn!("error setting window icon: {}", e)
        }


        let now = Instant::now();
        unsafe {
            let _ = GRAPHICS.set(graphics);
            let _ = RENDER_EVENT_RECEIVER.set(render_event_receiver);
            let _ = GAME_EVENT_SENDER.set(gane_event_sender);

            #[cfg(target_os = "windows")] 
            glfw::ffi::glfwSetWindowSizeCallback(window.window.window_ptr(), Some(RESIZE_WINDOW));
            #[cfg(target_os = "windows")] 
            glfw::ffi::glfwSetWindowPosCallback(window.window.window_ptr(), Some(REPOSITION_WINDOW));


            // extern "system" fn gl_callback(src:u32, t:u32, id:u32, severity:u32, len:i32, msg: *const i8, p: *mut c_void) {
            //     let e = unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string() };

            //     error!(
            //         "gl error! {e}\nsrc:{src}\ntype:{t:x}\nid:{id}\nseverity:{severity}"
            //     )

            // }
            gl::Enable(gl::DEBUG_OUTPUT);
            // gl::DebugMessageCallback(gl_callback, 0u8 as *const c_void);


        }

        Self {
            window,
            // graphics,
            // render_event_receiver,
            window_event_receiver,
            // game_event_sender: gane_event_sender, 
            window_size: WindowSizeHelper::default(),

            
            #[cfg(feature="bass_audio")] 
            bass,

            frametime_timer: now,
            input_timer: now,
        }
    }

    pub async fn run(&mut self) {
        // fire event so things get moved around correctly
        let settings = get_settings!().clone();
        set_window_size(settings.window_size.into());

        self.window_size.init().await;

        // resize window
        self.window.window.set_size(self.window_size.x as i32, self.window_size.y as i32);

        if settings.raw_mouse_input {
            self.window.window.set_raw_mouse_motion(true);
        }


        macro_rules! close_window {
            (self) => {
                self.window.window.set_should_close(true);
                unsafe {
                    GAME_EVENT_SENDER.get_mut().unwrap().ignite(GameEvent::WindowClosed);
                }
                return;
            }
        }
        
        loop {

            // poll window events
            while let Some(e) = self.window.poll_event() {
                if e.close_args().is_some() { close_window!(self); }

                if let Some(axis) = e.controller_axis_args() {
                    let j_id = get_joystick_id(axis.id);
                    let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
                    unsafe {
                        GAME_EVENT_SENDER.get_mut().unwrap().ignite(GameEvent::ControllerEvent(e, name));
                    }
                    continue
                }
                
                if let Some(Button::Controller(cb)) = e.button_args().map(|b|b.button) {
                    // debug!("press: c: {}, b: {}", cb.id, cb.button);

                    let j_id = get_joystick_id(cb.id);
                    let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
                    unsafe {
                        GAME_EVENT_SENDER.get_mut().unwrap().ignite(GameEvent::ControllerEvent(e, name));
                    }
                    continue;
                }

                if let Event::Input(Input::FileDrag(FileDrag::Drop(d)), _) = e {
                    unsafe {
                        GAME_EVENT_SENDER.get_mut().unwrap().ignite(GameEvent::DragAndDrop(d));
                    }
                    continue
                }

                unsafe {
                    GAME_EVENT_SENDER.get_mut().unwrap().ignite(GameEvent::WindowEvent(e));
                }
            }

            // check render-side events
            if let Ok(event) = self.window_event_receiver.try_recv() {
                match event {
                    WindowEvent::ShowCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Normal),
                    WindowEvent::HideCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Hidden),
                    WindowEvent::RequestAttention => self.window.window.request_attention(),
                    WindowEvent::SetRawInput(val) => self.window.window.set_raw_mouse_motion(val),
                    WindowEvent::SetClipboard(val) => self.window.window.set_clipboard_string(&val),
                    WindowEvent::CloseGame => { close_window!(self); },

                    WindowEvent::TakeScreenshot(fuze) => {
                        let width = self.window_size.x as usize;
                        let height = self.window_size.y as usize;

                        let data_size = 3 * width * height;
                        let mut window_data:Vec<u8> = vec![0; data_size];
                        let window_data2 = window_data.as_mut_slice().as_mut_ptr() as *mut std::ffi::c_void;

                        unsafe {
                            gl::ReadPixels(
                                0, 
                                0, 
                                width as i32, 
                                width as i32, 
                                gl::RGB, 
                                gl::UNSIGNED_BYTE, 
                                window_data2
                            );
                        }

                        // screenshot is upside down for some reason
                        let mut window_data2 = Vec::new();
                        for i in (0..window_data.len()).step_by(3 * width).rev() {
                            window_data2.extend(window_data[i..i + 3 * width].iter());
                        }
                        
                        // send it off
                        fuze.ignite((window_data2, width as u32, height as u32));
                    }
                }
            }

            // increment input frametime stuff
            let frametime = (self.input_timer.duration_and_reset() * 100.0).floor() as u32;
            INPUT_FRAMETIME.fetch_max(frametime, SeqCst);
            INPUT_COUNT.fetch_add(1, SeqCst);

            // actually render
            self.render().await;
            tokio::task::yield_now().await;
        }
    }
    
    async fn render(&mut self) {
        // unsafe {
        //     if !RENDER_EVENT_RECEIVER.get().unwrap().updated() { return }
        // }

        render(
            self.window.window.window_ptr(),
            RenderArgs {
                ext_dt: 0.0,
                window_size: self.window.size().into(),
                draw_size:   self.window.draw_size().into(),
            },
            &mut self.frametime_timer
        );
    }
}


fn render(window: *mut glfw::ffi::GLFWwindow, args: RenderArgs, frametime: &mut Instant) {
    unsafe {
        if let Ok(_) = NEW_RENDER_DATA_AVAILABLE.compare_exchange(true, false, Acquire, Relaxed) {
            match RENDER_EVENT_RECEIVER.get_mut().unwrap().read() {
                TatakuRenderEvent::None => {},
                TatakuRenderEvent::Draw(data) => {
                    let graphics = graphics();

                    let frametime = (frametime.duration_and_reset() * 100.0).floor() as u32;
                    RENDER_FRAMETIME.fetch_max(frametime, SeqCst);
                    RENDER_COUNT.fetch_add(1, SeqCst);

                    // TODO: use this for snipping
                    {
                        // // actually draw everything now
                        // let mut orig_c = self.graphics.draw_begin(args.viewport());

                        // graphics::clear(GFX_CLEAR_COLOR.into(), &mut self.graphics);
                        // for i in self.render_queue.iter_mut() {
                        //     let mut drawstate_changed = false;
                        //     let c = if let Some(ic) = i.get_context() {
                        //         drawstate_changed = true;
                        //         // debug!("ic: {:?}", ic);
                        //         self.graphics.draw_end();
                        //         self.graphics.draw_begin(args.viewport());
                        //         self.graphics.use_draw_state(&ic.draw_state);
                        //         ic
                        //     } else {
                        //         orig_c
                        //     };
                            
                        //     // self.graphics.use_draw_state(&c.draw_state);
                        //     if i.get_spawn_time() == 0 {i.set_spawn_time(elapsed)}
                        //     i.draw(&mut self.graphics, c);

                        //     if drawstate_changed {
                        //         self.graphics.draw_end();
                        //         orig_c = self.graphics.draw_begin(args.viewport());
                        //         self.graphics.use_draw_state(&orig_c.draw_state);
                        //     }
                        // }
                        // self.graphics.draw_end();
                    }

                    // TODO: dont use this for snipping
                    let c = graphics.draw_begin(args.viewport());
                    graphics::clear(GFX_CLEAR_COLOR.into(), graphics);
                    
                    for i in data.iter() {
                        i.draw(graphics, c);
                    }
                    if let Some(q) = CURSOR_RENDER_QUEUE.get() {
                        if let Ok(mut q) = q.try_lock() {
                            for i in q.read().iter() {
                                i.draw(graphics, c);
                            }
                        }
                    }
                    
                    graphics.draw_end();

                    // loop {
                    //     let e = gl::GetError();
                    //     if e == gl::NO_ERROR { break }
                    //     println!("gl error: {e}");
                    // }

                    glfw::ffi::glfwSwapBuffers(window);
                }
            }
        }

    }
}



pub enum TatakuRenderEvent {
    None,
    Draw(Vec<Box<dyn Renderable>>),
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
}

fn get_joystick_id(id: u32) -> glfw::JoystickId {
    use glfw::JoystickId::*;
    match id {
        0  => Joystick1,
        1  => Joystick2,
        2  => Joystick3,
        3  => Joystick4,
        4  => Joystick5,
        5  => Joystick6,
        6  => Joystick7,
        7  => Joystick8,
        8  => Joystick9,
        9  => Joystick10,
        10 => Joystick11,
        11 => Joystick12,
        12 => Joystick13,
        13 => Joystick14,
        14 => Joystick15,
        15 => Joystick16,
        _ => panic!("unknown joystick id: {}", id)
    }
}

// callbacks for windows because windows is bad
#[cfg(target_os = "windows")] 
pub static RESIZE_WINDOW:extern "C" fn(window: *mut glfw::ffi::GLFWwindow, i32, i32) = {
    extern "C" fn actual_callback(window: *mut glfw::ffi::GLFWwindow, w:i32, h:i32) {

        // generate a window event
        let draw_size = unsafe {
            let mut width = 0;
            let mut height = 0;
            glfw::ffi::glfwGetFramebufferSize(window, &mut width, &mut height);
            [width as u32, height as u32]
        };
        let window_size = [w as f64, h as f64];

        unsafe {
            GAME_EVENT_SENDER.get_mut().unwrap().ignite(
                GameEvent::WindowEvent(
                    Event::Input(
                        Input::Resize(ResizeArgs {
                            window_size,
                            draw_size,
                        }), 
                        None
                    )
                )
            );
        }

        let args = RenderArgs { 
            ext_dt: 0.0, 
            window_size,
            draw_size
        };

        let mut timer = Instant::now();

        // re-render
        render(window, args, &mut timer);
    }
    actual_callback
};

#[cfg(target_os = "windows")] 
pub static REPOSITION_WINDOW:extern "C" fn(window: *mut glfw::ffi::GLFWwindow, i32, i32) = {
    extern "C" fn actual_callback(window: *mut glfw::ffi::GLFWwindow, _x:i32, _y:i32) {
        let draw_size = unsafe {
            let mut width = 0;
            let mut height = 0;
            glfw::ffi::glfwGetFramebufferSize(window, &mut width, &mut height);
            [width as u32, height as u32]
        };
        let window_size = unsafe {
            let mut width = 0;
            let mut height = 0;
            glfw::ffi::glfwGetWindowSize(window, &mut width, &mut height);
            [width as f64, height as f64]
        };

        let args = RenderArgs { 
            ext_dt: 0.0, 
            window_size,
            draw_size
        };

        let mut timer = Instant::now();
        render(window, args, &mut timer);
    }
    actual_callback
};