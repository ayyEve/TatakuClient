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
static GAME_EVENT_SENDER: OnceCell<tokio::sync::mpsc::Sender<GameEvent>> = OnceCell::const_new();

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

#[cfg(feature="desktop")]
type InnerWindow = glfw_window::GlfwWindow;

#[cfg(feature="mobile")]
type InnerWindow = glutin_window::GlutinWindow;


pub struct GameWindow {
    pub window: InnerWindow,

    window_event_receiver: Receiver<WindowEvent>,

    frametime_timer: Instant,
    input_timer: Instant,

    window_size: WindowSizeHelper,

    frametime_logger: FrameTimeLogger
}

impl GameWindow {
    pub fn start(render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>, gane_event_sender: tokio::sync::mpsc::Sender<GameEvent>) -> Self {
        let opengl = OpenGL::V4_5;
        let window_size = WindowSizeHelper::new();
        
        // #[cfg(feature="desktop")] {
            let mut window:InnerWindow = WindowSettings::new("Tataku!", [window_size.x as u32, window_size.y as u32])
                .graphics_api(opengl)
                // .resizable(false)
                // .fullscreen(true) // this doesnt work?
                // .samples(32) // not sure if this actually works or not
                .build()
                .expect("Error creating window");
            // window.window.set_cursor_mode(glfw::CursorMode::Hidden);
        // }
        
        // #[cfg(feature="mobile")] {
        //     let mut window = glutin_window::GlutinWindow::new(&WindowSettings::new())
        // }


        let graphics = GlGraphics::new(opengl);
        debug!("done graphics");


        // pre-load fonts
        get_font();
        get_fallback_font();
        get_font_awesome();
        debug!("done fonts");

        
        let (window_event_sender, window_event_receiver) = sync_channel(10);
        WINDOW_EVENT_QUEUE.set(window_event_sender).ok().expect("bad");
        debug!("done texture load queue");
        
        
        #[cfg(feature="desktop")] {
            AudioManager::init_audio().expect("error initializing audio");

            // set window icon
            match image::open("resources/icon-small.png") {
                Ok(img) => {
                    window.window.set_icon(vec![img.into_rgba8()]);
                }
                Err(e) => warn!("error setting window icon: {}", e)
            }
        }
        
        #[cfg(feature="mobile")] {
            AudioManager::init_audio(0 as *mut std::ffi::c_void).expect("error initializing audio");
        }

        let now = Instant::now();
        unsafe {
            let _ = GRAPHICS.set(graphics);
            let _ = RENDER_EVENT_RECEIVER.set(render_event_receiver);
            let _ = GAME_EVENT_SENDER.set(gane_event_sender);

            #[cfg(target_os = "windows")] #[cfg(feature = "desktop")] {
                glfw::ffi::glfwSetWindowSizeCallback(window.window.window_ptr(), Some(RESIZE_WINDOW));
                glfw::ffi::glfwSetWindowPosCallback(window.window.window_ptr(), Some(REPOSITION_WINDOW));
            }


            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            extern "system" fn gl_callback(_src:u32, _t:u32, _id:u32, _severity:u32, _len:i32, msg: *const i8, _p: *mut std::ffi::c_void) {
                let e = unsafe { std::ffi::CStr::from_ptr(msg).to_string_lossy().to_string() };
                if e.starts_with("Buffer detailed info") { return }
                error!("gl: {e}")
            }
            gl::DebugMessageCallback(gl_callback, 0u8 as *const std::ffi::c_void);
        }

        Self {
            window,

            window_event_receiver,
            window_size,

            frametime_timer: now,
            input_timer: now,

            frametime_logger: FrameTimeLogger::new(),
        }
    }

    pub async fn run(&mut self) {
        // fire event so things get moved around correctly
        let settings = get_settings!().clone();
        GlobalValueManager::update(Arc::new(WindowSize(settings.window_size.into())));

        self.window_size.update();

        #[cfg(feature = "desktop")] {
            // resize window
            self.window.window.set_size(self.window_size.x as i32, self.window_size.y as i32);

            if settings.raw_mouse_input {
                self.window.window.set_raw_mouse_motion(true);
            }
        } 
        
        #[cfg(feature = "mobile")] {
            let size = [self.window_size.x as u32, self.window_size.y as u32];
            self.window.ctx.resize(size.into());
        }
        

        macro_rules! close_window {
            (self) => {
                #[cfg(feature = "desktop")] self.window.window.set_should_close(true);
                #[cfg(feature = "mobile")] self.window.set_should_close(true);
                let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowClosed);
                self.frametime_logger.write();
                return;
            }
        }
        
        loop {
            self.window_size.update();

            // poll window events
            while let Some(e) = self.window.poll_event() {
                if e.close_args().is_some() { close_window!(self); }

                #[cfg(feature="desktop")]
                if let Some(axis) = e.controller_axis_args() {
                    let j_id = get_joystick_id(axis.id);
                    let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
                    let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::ControllerEvent(e, name));
                    continue
                }
                
                #[cfg(feature="desktop")]
                if let Some(piston::input::Button::Controller(cb)) = e.button_args().map(|b|b.button) {
                    // debug!("press: c: {}, b: {}", cb.id, cb.button);

                    let j_id = get_joystick_id(cb.id);
                    let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
                    let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::ControllerEvent(e, name));
                    
                    continue;
                }

                if let Event::Input(Input::FileDrag(FileDrag::Drop(d)), _) = e {
                    let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::DragAndDrop(d));
                    continue
                }

                let _ = GAME_EVENT_SENDER.get().unwrap().try_send(GameEvent::WindowEvent(e));
            }

            // check render-side events
            if let Ok(event) = self.window_event_receiver.try_recv() {
                #[cfg(feature="desktop")]
                match event {
                    WindowEvent::ShowCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Normal),
                    WindowEvent::HideCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Hidden),
                    WindowEvent::RequestAttention => self.window.window.request_attention(),
                    WindowEvent::SetRawInput(val) => self.window.window.set_raw_mouse_motion(val),
                    WindowEvent::SetClipboard(val) => self.window.window.set_clipboard_string(&val),
                    WindowEvent::CloseGame => { close_window!(self); },
                    WindowEvent::TakeScreenshot(fuze) => self.screenshot(fuze),
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
        let args = RenderArgs {
            ext_dt: 0.0,
            window_size: self.window.size().into(),
            draw_size:   self.window.draw_size().into(),
        };
        
        #[cfg(feature="desktop")]
        let window = self.window.window.window_ptr();
        
        #[cfg(feature="mobile")]
        let window = &mut self.window;

        render(window, args, &mut self.frametime_timer);
    }

    fn screenshot(&self, fuze: Fuze<(Vec<u8>, u32, u32)>) {
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

}


#[cfg(feature="desktop")]
type WindowPtr = *mut glfw::ffi::GLFWwindow;

#[cfg(feature="mobile")]
type WindowPtr<'a> = &'a mut InnerWindow;

fn render(window: WindowPtr, args: RenderArgs, frametime: &mut Instant) {
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
                    
                    #[cfg(feature="desktop")]
                    glfw::ffi::glfwSwapBuffers(window);

                    
                    #[cfg(feature="mobile")]
                    window.swap_buffers();
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

#[cfg(feature="desktop")] #[cfg(target_os = "windows")] 
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

        let _ = GAME_EVENT_SENDER.get().unwrap().try_send(
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


#[cfg(feature="desktop")] #[cfg(target_os = "windows")] 
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
