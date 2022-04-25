use crate::prelude::*;
use glfw_window::GlfwWindow as AppWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    input::*, 
    window::WindowSettings,
    Window
};


/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;

pub static WINDOW_EVENT_QUEUE: OnceCell<SyncSender<RenderSideEvent>> = OnceCell::const_new();


pub struct GameWindow {
    pub window: AppWindow,
    pub graphics: GlGraphics,

    game_event_sender: MultiFuze<GameEvent>,
    render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>,
    window_event_receiver: Receiver<RenderSideEvent>,

    #[cfg(feature="bass_audio")]
    #[allow(dead_code)]
    /// needed to prevent bass from deinitializing
    bass: bass_rs::Bass,
}

impl GameWindow {
    pub fn start(render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>, gane_event_sender: MultiFuze<GameEvent>) -> Self {
        let window_size = Settings::window_size();

        let opengl = OpenGL::V3_0;
        let mut window: AppWindow = WindowSettings::new("Tataku!", [window_size.x, window_size.y])
            .graphics_api(opengl)
            .resizable(false)
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

        
        info!("done texture load queue");

        let (window_event_sender, window_event_receiver) = sync_channel(10);
        WINDOW_EVENT_QUEUE.set(window_event_sender).ok().expect("bad");
        
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
            Ok(img) => {
                window.window.set_icon(vec![img.into_rgba8()]);
                info!("window icon set");
            }
            Err(e) => {
                info!("error setting window icon: {}", e);
            }
        }


        Self {
            window,
            graphics,
            render_event_receiver,
            window_event_receiver,
            game_event_sender: gane_event_sender, 

            
            #[cfg(feature="bass_audio")] 
            bass,
        }
    }

    pub async fn run(&mut self) {
        loop {
            while let Some(e) = self.window.poll_event() {

                if let Some(axis) = e.controller_axis_args() {
                    let j_id = get_joystick_id(axis.id);
                    let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
                    self.game_event_sender.ignite(GameEvent::ControllerEvent(e, name));
                    continue
                }
                
                if let Some(Button::Controller(cb)) = e.button_args().map(|b|b.button) {
                    // debug!("press: c: {}, b: {}", cb.id, cb.button);

                    let j_id = get_joystick_id(cb.id);
                    let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
                    self.game_event_sender.ignite(GameEvent::ControllerEvent(e, name));
                    
                    continue;
                }

                self.game_event_sender.ignite(GameEvent::WindowEvent(e));
            }

            // check render-side events
            if let Ok(event) = self.window_event_receiver.try_recv() {
                match event {
                    RenderSideEvent::ShowCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Normal),
                    RenderSideEvent::HideCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Hidden),
                    RenderSideEvent::CloseGame => {
                        self.window.window.set_should_close(true);
                        self.game_event_sender.ignite(GameEvent::WindowClosed);
                        return
                    },
                }
            }

            self.render().await;

            tokio::task::yield_now().await;
        }

    }
    
    async fn render(&mut self) {
        if !self.render_event_receiver.updated() {return}

        match self.render_event_receiver.read() {
            TatakuRenderEvent::None => {},
            TatakuRenderEvent::Draw(data) => {
                let args = RenderArgs {
                    ext_dt: 0.0,
                    window_size: self.window.size().into(),
                    draw_size: self.window.draw_size().into(),
                };

                // TODO: use this for snipping
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


                // TODO: dont use this for snipping

                let c = self.graphics.draw_begin(args.viewport());
                graphics::clear(GFX_CLEAR_COLOR.into(), &mut self.graphics);
                for i in data.iter() {
                    i.draw(&mut self.graphics, c);
                }
                if let Some(q) = CURSOR_RENDER_QUEUE.get() {
                    for i in q.lock().await.read().iter() {
                        i.draw(&mut self.graphics, c);
                    }
                }
                self.graphics.draw_end();
                self.window.swap_buffers();

            },
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




pub enum RenderSideEvent {
    ShowCursor,
    HideCursor,
    CloseGame,
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