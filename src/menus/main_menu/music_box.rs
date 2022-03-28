use crate::prelude::*;

// draw buttons
const CONTROL_BUTTONS: &[Option<FontAwesome>] = &[
    Some(FontAwesome::Backward),
    Some(FontAwesome::Backward_Step),
    None,
    Some(FontAwesome::Play), // Some(FontAwesome::Pause), //  detect for this
    None,
    Some(FontAwesome::Forward_Step),
    Some(FontAwesome::Forward),
];

const MUSIC_BOX_PADDING:Vector2 = Vector2::new(5.0, 5.0);
const CONTROL_BUTTON_SIZE:u32 = 30;
const CONTROL_BUTTON_MARGIN_WHEN_NONE:f64 = 15.0;
/// x margin between buttons
const CONTROL_BUTTON_X_MARGIN:f64 = 10.0;

const CONTROL_BUTTON_PADDING:Vector2 = Vector2::new(15.0, 15.0);
const Y_BOTTOM_PADDING:f64 = 0.0;
const X_PADDING:f64 = 0.0;

const SKIP_AMOUNT:f64 = 500.0; // half a second?

#[derive(ScrollableGettersSetters)]
pub struct MusicBox {
    pos: Vector2, // should be bottom right
    size: Vector2,
    hover: bool,
    mouse_pos: Vector2,

    texts: Vec<Text>,
    actions: Vec<FontAwesome>,

    next_pending: AtomicBool,
    prev_pending: AtomicBool,
}
impl MusicBox {
    pub fn new() -> Self {
        // this is a big mess

        let mut size = Vector2::zero();
        let mut pos = Vector2::new(X_PADDING, Settings::window_size().y);

        // setup buttons
        let mut texts = Vec::new();
        let mut actions = Vec::new();
        let mut btn_pos = pos + Vector2::x_only(CONTROL_BUTTON_PADDING.x); // add initial left-side pad
        let font_awesome = get_font_awesome();
        for button in CONTROL_BUTTONS {
            if let Some(c) = button {
                actions.push(*c);

                let text = Text::new(
                    Color::WHITE,
                    0.0,
                    btn_pos + MUSIC_BOX_PADDING,
                    CONTROL_BUTTON_SIZE,
                    format!("{}", c.get_char()),
                    font_awesome.clone()
                );

                let t_size = text.measure_text();
                btn_pos.x += t_size.x + CONTROL_BUTTON_PADDING.x * 2.0 + CONTROL_BUTTON_X_MARGIN;
                size.y = size.y.max(t_size.y);
                texts.push(text);
            } else {
                btn_pos.x += CONTROL_BUTTON_MARGIN_WHEN_NONE;
            }
        }
        size.x = btn_pos.x - (pos.x + CONTROL_BUTTON_X_MARGIN + CONTROL_BUTTON_PADDING.x);

        let size = 
            MUSIC_BOX_PADDING * 2.0 // add padding
            + size // button sizes
            + Vector2::y_only(CONTROL_BUTTON_PADDING.y * 2.0) // control button border padding
            ;

        // update text's y pos
        pos.y -= size.y + Y_BOTTOM_PADDING; // bottom padding;
        for i in texts.iter_mut() {
            i.current_pos.y = pos.y + MUSIC_BOX_PADDING.y + CONTROL_BUTTON_PADDING.y;
        }

        Self {
            size, 
            pos,
            hover: false,
            mouse_pos: Vector2::zero(),
            actions,
            texts,

            next_pending: AtomicBool::new(false),
            prev_pending: AtomicBool::new(false),
        }
    }
    pub fn get_next_pending(&self) -> bool {
        let val = &self.next_pending;

        if val.load(SeqCst) {
            val.store(false, SeqCst);
            true
        } else {
            false
        }
    }
    pub fn get_prev_pending(&self) -> bool {
        let val = &self.prev_pending;
        if val.load(SeqCst) {
            val.store(false, SeqCst);
            true
        } else {
            false
        }
    }


    fn pause_or_resume(&self) {
        #[cfg(feature = "bass_audio")] {
            if let Some(s) = Audio::get_song() {
                if let Ok(state) = s.get_playback_state() {
                    match state {
                        PlaybackState::Stopped | PlaybackState::Stalled => if let Err(_) = s.play(true) {
                            self.next()
                        },
                        PlaybackState::Playing => s.pause().expect("unable to pause?"),
                        PlaybackState::Paused | PlaybackState::PausedDevice => s.play(false).expect("unable to play?"),
                    }
                }
            }
        }
    }
    fn stop(&self) {
        #[cfg(feature = "bass_audio")]
        if let Some(s) = Audio::get_song() {
            let _ = s.stop();
        }
    }
    fn skip_ahead(&self) {
        #[cfg(feature = "bass_audio")]
        if let Some(s) = Audio::get_song() {
            let current_pos = s.get_position().unwrap_or_default();
            let _ = s.set_position(current_pos + SKIP_AMOUNT);
        }
    }
    fn skip_behind(&self) {
        #[cfg(feature = "bass_audio")]
        if let Some(s) = Audio::get_song() {
            let current_pos = s.get_position().unwrap_or_default();
            let _ = s.set_position((current_pos - SKIP_AMOUNT).max(0.0));
        }
    }
    fn next(&self) {
        self.next_pending.store(true, SeqCst);
    }
    fn previous(&self) {
        self.prev_pending.store(true, SeqCst);
    }
}
impl ScrollableItem for MusicBox {
    fn on_mouse_move(&mut self, p:Vector2) {
        self.check_hover(p);
        self.mouse_pos = p;
    }

    fn on_click(&mut self, pos:Vector2, _button:MouseButton, _mods:KeyModifiers) -> bool {
        for (i, text) in self.texts.iter().enumerate() {
            let t_size = text.measure_text();
            
            // make bounding box
            let rect = Rectangle::bounds_only(
                text.current_pos - CONTROL_BUTTON_PADDING, 
                t_size + CONTROL_BUTTON_PADDING * 2.0,
            );

            if rect.contains(pos) {
                match self.actions.get(i) {
                    Some(&FontAwesome::Play)
                    | Some(&FontAwesome::Circle_Play) => self.pause_or_resume(),
                    
                    Some(&FontAwesome::Stop)
                    | Some(&FontAwesome::Circle_Stop) => self.stop(),

                    Some(&FontAwesome::Backward) => self.previous(),
                    Some(&FontAwesome::Forward) => self.next(),
                    Some(&FontAwesome::Backward_Step) => self.skip_behind(),
                    Some(&FontAwesome::Forward_Step) => self.skip_ahead(),

                    _ => warn!("unknown action"),
                }
            }
        }
        
        self.hover
    }

    // fn on_click_release(&mut self, _pos:Vector2, _button:MouseButton) {}

    // fn on_key_press(&mut self, _key:Key, _mods:KeyModifiers) -> bool {false}

    // fn update(&mut self) {}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list: &mut Vec<Box<dyn Renderable>>) {
        let draw_pos = self.pos + pos_offset;

        // // draw bg
        // list.push(Box::new(Rectangle::new(
        //     Color::HOT_PINK.alpha(0.1),
        //     parent_depth,
        //     draw_pos,
        //     self.size,
        //     None, // Some(Border::new(Color::LIGHT_BLUE, 2.0))
        // ).shape(Shape::Round(5.0, 10))));


        // draw buttons
        for text in self.texts.iter() {
            let mut text = text.clone();
            text.depth = parent_depth;
            text.current_pos += pos_offset;
            
            let t_size = text.measure_text();
            
            // make bounding box
            let mut rect = Rectangle::new(
                Color::new(1.0, 1.0, 1.0, 0.1),
                parent_depth,
                text.current_pos - CONTROL_BUTTON_PADDING, 
                t_size + CONTROL_BUTTON_PADDING * 2.0,
                None, //Some(Border::new(Color::BLACK, 1.2))
            ).shape(Shape::Round(5.0, 10));

            if rect.contains(self.mouse_pos) {
                rect.current_color.a = 0.2;
            }
            // rect.border.as_mut().unwrap().color.a = 0.8;

            // add rect
            list.push(Box::new(rect));

            // add text after rect
            list.push(Box::new(text));
        }
    }
}