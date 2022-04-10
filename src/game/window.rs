use image::RgbaImage;

use crate::prelude::*;
use glfw_window::GlfwWindow as AppWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{
    input::*, 
    event_loop::*, 
    window::WindowSettings,
    RenderEvent
};

/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;
pub static TEXTURE_LOAD_QUEUE: OnceCell<SyncSender<(LoadImage, SyncSender<TatakuResult<Arc<Texture>>>)>> = OnceCell::const_new();

pub static WINDOW_EVENT_QUEUE: OnceCell<SyncSender<RenderSideEvent>> = OnceCell::const_new();


pub struct GameWindow {
    pub window: AppWindow,
    pub graphics: GlGraphics,

    game_event_sender: MultiFuze<GameEvent>,
    render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>,
    window_event_receiver: Receiver<RenderSideEvent>,
    texture_load_receiver: Receiver<(LoadImage, SyncSender<TatakuResult<Arc<Texture>>>)>,
    

    #[cfg(feature="bass_audio")]
    #[allow(dead_code)]
    /// needed to prevent bass from deinitializing
    bass: bass_rs::Bass,
}

impl GameWindow {
    pub fn start(render_event_receiver: TripleBufferReceiver<TatakuRenderEvent>, gane_event_sender: MultiFuze<GameEvent>) -> Self {
        let window_size = Settings::window_size();

        let opengl = OpenGL::V3_2;
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

        
        let (texture_load_sender, texture_load_receiver) = sync_channel(2_000);
        TEXTURE_LOAD_QUEUE.set(texture_load_sender).ok().expect("bad");
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
            texture_load_receiver,
            window_event_receiver,
            game_event_sender: gane_event_sender, 

            
            #[cfg(feature="bass_audio")] 
            bass,
        }
    }

    pub fn run(&mut self) {
        let mut events = Events::new(EventSettings::new());
        events.set_ups_reset(0);

        // used to keep textures from dropping off the main thread
        let mut image_data = Vec::new();

        {
            // input and rendering thread times
            let settings = get_settings!();
            events.set_max_fps(settings.fps_target);
            events.set_ups(settings.update_target);
        }

        'render_loop: while let Some(e) = events.next(&mut self.window) {
            if let Some(_args) = e.update_args() {
                if let Ok((method, on_done)) = self.texture_load_receiver.try_recv() {
                    let settings = opengl_graphics::TextureSettings::new();

                    macro_rules! send_tex {
                        ($tex:expr) => {{
                            let tex = Arc::new($tex);
                            image_data.push(tex.clone());
                            on_done.send(Ok(tex)).expect("uh oh");
                        }}
                    }

                    match method {
                        LoadImage::Path(path) => {
                            match Texture::from_path(path, &settings) {
                                Ok(t) => send_tex!(t),
                                Err(e) => on_done.send(Err(TatakuError::String(e))).expect("uh oh"),
                            };
                        },
                        LoadImage::Image(data) => {
                            send_tex!(Texture::from_image(&data, &settings))
                        },
                        LoadImage::Font(font, size) => {
                            let px = size.0;

                            // let mut textures = font.textures.write();
                            let mut characters = font.characters.write();


                            for (&char, _codepoint) in font.font.chars() {

                                // TODO: load as one big image per-font
                                
                                // generate glyph data
                                let (metrics, bitmap) = font.font.rasterize(char, px);

                                let mut data = Vec::new();
                                // bitmap is a vec of grayscale pixels
                                // we need to turn that into rgba bytes

                                bitmap.into_iter().for_each(|gray| {
                                    data.push(255); // r
                                    data.push(255); // g
                                    data.push(255); // b
                                    data.push(gray); // a
                                });

                                // convert to image
                                let data = image::RgbaImage::from_vec(metrics.width as u32, metrics.height as u32, data).unwrap();

                                // load in opengl
                                let texture = Arc::new(Texture::from_image(&data, &settings));

                                // setup data
                                let char_data = CharData {
                                    texture,
                                    pos: Vector2::zero(),
                                    size: Vector2::new(metrics.width as f64, metrics.height as f64),
                                    metrics,
                                };

                                // insert data
                                characters.insert((size, char), char_data);
                            }

                            on_done.send(Err(TatakuError::String(String::new()))).expect("uh oh");
                        }
                    }

                    info!("done loading tex");
                }

                // drop textures that only have a reference here (ie, dropped everywhere else)
                image_data.retain(|i| {
                    Arc::strong_count(i) > 1
                });

                // check render-side events
                if let Ok(event) = self.window_event_receiver.try_recv() {
                    match event {
                        RenderSideEvent::ShowCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Normal),
                        RenderSideEvent::HideCursor => self.window.window.set_cursor_mode(glfw::CursorMode::Hidden),
                        RenderSideEvent::CloseGame => {
                            self.window.window.set_should_close(true);
                            break 'render_loop;
                        },
                    }
                }

                continue
            } //{self.update(args.dt*1000.0)}

            if let Some(args) = e.render_args() {
                // info!("window render event");
                self.render(args);
                continue
            }

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

            // trace!("sending event");
            self.game_event_sender.ignite(GameEvent::WindowEvent(e));



            // self.input_manager.handle_events(e.clone(), &mut self.window);
            // if let Some(args) = e.update_args() {self.update(args.dt*1000.0)}
            // if let Some(args) = e.render_args() {self.render(args)}
            // // if let Some(Button::Keyboard(_)) = e.press_args() {self.input_update_display.increment()}



            // e.resize(|args| debug!("Resized '{}, {}'", args.window_size[0], args.window_size[1]));
        }

        self.game_event_sender.ignite(GameEvent::WindowClosed);
    }
    
    fn render(&mut self, args: RenderArgs) {
        if !self.render_event_receiver.updated() {return}

        match self.render_event_receiver.read() {
            TatakuRenderEvent::None => {},
            TatakuRenderEvent::Draw(data) => {
                // info!("draw");

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

                self.graphics.draw(args.viewport(), |c, g| {
                    graphics::clear(GFX_CLEAR_COLOR.into(), g);
                    for i in data.iter() {
                        i.draw(g, c);
                    }
                    if let Some(q) = CURSOR_RENDER_QUEUE.get() {
                        q.lock().read().iter().for_each(|i| {
                            i.draw(g, c);
                        })
                    }
                });
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



pub fn load_texture<P: AsRef<Path>>(path: P) -> TatakuResult<Arc<Texture>> {
    let path = path.as_ref().to_string_lossy().to_string();
    info!("loading tex {}", path);

    let (sender, receiver) = sync_channel(2);
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Path(path), sender)).expect("no?");

    if let Ok(t) = receiver.recv() {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}
pub fn load_texture_data(data: RgbaImage) -> TatakuResult<Arc<Texture>> {
    info!("loading tex data");
    let (sender, receiver) = sync_channel(2);
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Image(data), sender)).expect("no?");

    if let Ok(t) = receiver.recv() {
        t
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}

pub fn load_font_data(font: Font2, size:FontSize) -> TatakuResult<()> {
    // info!("loading font char ('{ch}',{size})");
    let (sender, receiver) = sync_channel(2);
    TEXTURE_LOAD_QUEUE.get().unwrap().send((LoadImage::Font(font, size), sender)).expect("no?");

    if let Ok(t) = receiver.recv() {
        t.map(|_|())
    } else {
        Err(TatakuError::String("idk".to_owned()))
    }
}


pub enum LoadImage {
    Path(String),
    Image(RgbaImage),
    Font(Font2, FontSize),
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