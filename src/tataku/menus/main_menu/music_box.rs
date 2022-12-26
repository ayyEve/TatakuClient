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

const SKIP_AMOUNT:f32 = 500.0; // half a second?

const PROGRESSBAR_HEIGHT:f64 = 5.0;
const PROGRESSBAR_YPAD:f64 = 2.0;

const PRIMARY_COLOR:Color = Color::WHITE;
const SECONDARY_COLOR:Color = Color::new(1.0, 1.0, 1.0, 0.1);

#[derive(ScrollableGettersSetters)]
pub struct MusicBox {
    pos: Vector2, // should be bottom right
    size: Vector2,
    hover: bool,
    mouse_pos: Vector2,

    song_time: f32,
    song_duration: f32,

    texts: Vec<Text>,
    actions: Vec<FontAwesome>,

    next_pending: AtomicBool,
    prev_pending: AtomicBool,
}
impl MusicBox {
    pub async fn new() -> Self {
        // this is a big mess
        let window_size = WindowSize::get();
        let mut size = Vector2::zero();
        let mut pos = Vector2::new(X_PADDING, window_size.y);

        // setup buttons
        let mut texts = Vec::new();
        let mut actions = Vec::new();
        let mut btn_pos = pos + Vector2::with_x(CONTROL_BUTTON_PADDING.x); // add initial left-side pad
        let font_awesome = get_font_awesome();
        for button in CONTROL_BUTTONS {
            if let Some(c) = button {
                actions.push(*c);

                let text = Text::new(
                    PRIMARY_COLOR,
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
            + Vector2::with_y(CONTROL_BUTTON_PADDING.y * 2.0) // control button border padding
            ;

        // update text's y pos
        pos.y -= size.y + Y_BOTTOM_PADDING + PROGRESSBAR_HEIGHT * 2.0; // bottom padding;
        for i in texts.iter_mut() {
            i.pos.y = pos.y + MUSIC_BOX_PADDING.y + CONTROL_BUTTON_PADDING.y;
        }

        Self {
            pos, 
            size, 
            hover: false, 
            mouse_pos: Vector2::zero(), 
            actions, 
            texts, 

            song_time: 0.0, 
            song_duration: 0.0, 

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

    pub fn update_song_time(&mut self, time: f32) {
        self.song_time = time;
    }
    pub fn update_song_duration(&mut self, time: f32) {
        self.song_duration = time;
    }

    fn pause_or_resume() {
        tokio::spawn(async {
            if let Some(s) = AudioManager::get_song().await {
                if s.is_stopped() { 
                    s.play(true); 
                } else if s.is_playing() {
                    s.pause()
                } else if s.is_paused() {
                    s.play(false);
                }
            }
        });
    }
    fn stop() {
        tokio::spawn(async {
            if let Some(s) = AudioManager::get_song().await {
                s.stop();
            }
        });
    }
    fn skip_ahead() {
        tokio::spawn(async {
            if let Some(s) = AudioManager::get_song().await {
                let current_pos = s.get_position();
                s.set_position(current_pos + SKIP_AMOUNT);
            }
        });
    }
    fn skip_behind() {
        tokio::spawn(async {
            if let Some(s) = AudioManager::get_song().await {
                let current_pos = s.get_position();
                s.set_position((current_pos - SKIP_AMOUNT).max(0.0));
            }
        });
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
                text.pos - CONTROL_BUTTON_PADDING, 
                t_size + CONTROL_BUTTON_PADDING * 2.0,
            );

            if rect.contains(pos) {
                match self.actions.get(i) {
                    Some(&FontAwesome::Play)
                    | Some(&FontAwesome::Circle_Play) => Self::pause_or_resume(),
                    
                    Some(&FontAwesome::Stop)
                    | Some(&FontAwesome::Circle_Stop) => Self::stop(),

                    Some(&FontAwesome::Backward) => self.previous(),
                    Some(&FontAwesome::Forward) => self.next(),
                    Some(&FontAwesome::Backward_Step) => Self::skip_behind(),
                    Some(&FontAwesome::Forward_Step) =>  Self::skip_ahead(),

                    _ => warn!("unknown action"),
                }
            }
        }
        
        if Rectangle::bounds_only(
            self.pos + Vector2::with_y(self.size.y + PROGRESSBAR_YPAD), 
            Vector2::new(self.size.x, PROGRESSBAR_HEIGHT)
        ).contains(pos) {
            let rel_x = (pos - self.pos).x;
            let pos = (rel_x / self.size.x) * self.song_duration as f64;
            
            tokio::spawn(async move {
                if let Some(song) = AudioManager::get_song().await {
                    song.set_position(pos as f32);
                }
            });
        }

        self.hover
    }

    fn update(&mut self) {}

    fn draw(&mut self, _args:RenderArgs, pos_offset:Vector2, parent_depth:f64, list: &mut RenderableCollection) {
        // let draw_pos = self.pos + pos_offset;

        // draw buttons
        for mut text in self.texts.clone() {
            text.depth = parent_depth;
            text.pos += pos_offset;
            
            let t_size = text.measure_text();
            
            // make bounding box
            let mut rect = Rectangle::new(
                SECONDARY_COLOR.alpha(0.1),
                parent_depth,
                text.pos - CONTROL_BUTTON_PADDING, 
                t_size + CONTROL_BUTTON_PADDING * 2.0,
                None, //Some(Border::new(Color::BLACK, 1.2))
            ).shape(Shape::Round(5.0, 10));

            if rect.contains(self.mouse_pos) {
                rect.color.a = 0.2;
            }
            // rect.border.as_mut().unwrap().color.a = 0.8;

            // add rect
            list.push(rect);

            // add text after rect
            list.push(text);
        }


        // draw progress bar
        list.push(Rectangle::new(
            PRIMARY_COLOR,
            parent_depth,
            self.pos + pos_offset + Vector2::with_y(self.size.y + PROGRESSBAR_YPAD),
            Vector2::new(self.size.x * (self.song_time / self.song_duration) as f64, PROGRESSBAR_HEIGHT),
            None
        ).shape(Shape::Round(2.0, 5)));
        // draw border after
        list.push(Rectangle::new(
            Color::TRANSPARENT_WHITE,
            parent_depth,
            self.pos + pos_offset + Vector2::with_y(self.size.y + PROGRESSBAR_YPAD),
            Vector2::new(self.size.x, PROGRESSBAR_HEIGHT),
            Some(Border::new(SECONDARY_COLOR, 1.2))
        ).shape(Shape::Round(2.0, 5)));

    }
}