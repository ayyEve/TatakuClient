use crate::prelude::*;

// draw buttons
const CONTROL_BUTTONS: &[Option<FontAwesome>] = &[
    Some(FontAwesome::Backward),
    Some(FontAwesome::Backward_Step),
    None,
    Some(FontAwesome::Pause), // Some(FontAwesome::Pause), //  detect for this
    None,
    Some(FontAwesome::Forward_Step),
    Some(FontAwesome::Forward),
];
const PLAY_INDEX:usize = 2;

const MUSIC_BOX_PADDING:Vector2 = Vector2::new(5.0, 5.0);
const CONTROL_BUTTON_SIZE:u32 = 30;
const CONTROL_BUTTON_MARGIN_WHEN_NONE:f32 = 15.0;
/// x margin between buttons
const CONTROL_BUTTON_X_MARGIN:f32 = 10.0;

const CONTROL_BUTTON_PADDING:Vector2 = Vector2::new(15.0, 15.0);
const Y_BOTTOM_PADDING:f32 = 0.0;
const X_PADDING:f32 = 0.0;

const SKIP_AMOUNT:f32 = 500.0; // half a second?

const PROGRESSBAR_HEIGHT:f32 = 5.0;
const PROGRESSBAR_YPAD:f32 = 2.0;

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
    song_paused: bool,

    texts: Vec<Text>,
    actions: Vec<FontAwesome>,

    event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,
}
impl MusicBox {
    pub async fn new(event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,) -> Self {
        // this is a big mess
        let window_size = WindowSize::get();
        let mut size = Vector2::ZERO;
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
                    btn_pos + MUSIC_BOX_PADDING,
                    CONTROL_BUTTON_SIZE as f32,
                    c.get_char().to_string(),
                    PRIMARY_COLOR,
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
            mouse_pos: Vector2::ZERO, 
            actions, 
            texts, 

            song_time: 0.0, 
            song_duration: 0.0, 
            song_paused: false,

            event_sender,
        }
    }

    pub fn update_song_time(&mut self, time: f32) {
        self.song_time = time;
    }
    pub fn update_song_duration(&mut self, time: f32) {
        self.song_duration = time;
    }
    pub fn update_song_paused(&mut self, paused: bool) {
        self.song_paused = paused;
    }

    fn pause_or_resume(&self) {
        let _ = self.event_sender.send(MediaControlHelperEvent::Toggle);
    }
    fn stop(&self) {
        let _ = self.event_sender.send(MediaControlHelperEvent::Stop);
    }
    fn skip_ahead(&self) {
        let _ = self.event_sender.send(MediaControlHelperEvent::SeekForwardBy(SKIP_AMOUNT));
    }
    fn skip_behind(&self) {
        let _ = self.event_sender.send(MediaControlHelperEvent::SeekBackwardBy(SKIP_AMOUNT));
    }
    fn next(&self) {
        let _ = self.event_sender.send(MediaControlHelperEvent::Next);
    }
    fn previous(&self) {
        let _ = self.event_sender.send(MediaControlHelperEvent::Previous);
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
                    | Some(&FontAwesome::Pause)
                    | Some(&FontAwesome::Circle_Play)
                    | Some(&FontAwesome::Circle_Pause) => self.pause_or_resume(),
                    
                    Some(&FontAwesome::Stop)
                    | Some(&FontAwesome::Circle_Stop) => self.stop(),

                    Some(&FontAwesome::Backward) => self.previous(),
                    Some(&FontAwesome::Forward) => self.next(),
                    Some(&FontAwesome::Backward_Step) => self.skip_behind(),
                    Some(&FontAwesome::Forward_Step) =>  self.skip_ahead(),

                    _ => warn!("unknown action"),
                }
            }
        }
        
        if Rectangle::bounds_only(
            self.pos + Vector2::with_y(self.size.y + PROGRESSBAR_YPAD), 
            Vector2::new(self.size.x, PROGRESSBAR_HEIGHT)
        ).contains(pos) {
            let rel_x = (pos - self.pos).x;
            let pos = (rel_x / self.size.x) * self.song_duration;
            
            tokio::spawn(async move {
                if let Some(song) = AudioManager::get_song().await {
                    song.set_position(pos);
                }
            });
        }

        self.hover
    }

    fn update(&mut self) {}

    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {

        // draw buttons
        for (i, mut text) in self.texts.clone().into_iter().enumerate() {
            text.pos += pos_offset;

            // if this is the play button, and the song is paused, use the play character
            if i == PLAY_INDEX && self.song_paused {
                text.text = FontAwesome::Play.get_char().to_string();
            }
            
            let t_size = text.measure_text();
            
            // make bounding box
            let mut rect = Rectangle::new(
                text.pos - CONTROL_BUTTON_PADDING, 
                t_size + CONTROL_BUTTON_PADDING * 2.0,
                SECONDARY_COLOR.alpha(0.1),
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
            self.pos + pos_offset + Vector2::with_y(self.size.y + PROGRESSBAR_YPAD),
            Vector2::new(self.size.x * (self.song_time / self.song_duration), PROGRESSBAR_HEIGHT),
            PRIMARY_COLOR,
            None
        ).shape(Shape::Round(2.0, 5)));
        
        // draw border after
        list.push(Rectangle::new(
            self.pos + pos_offset + Vector2::with_y(self.size.y + PROGRESSBAR_YPAD),
            Vector2::new(self.size.x, PROGRESSBAR_HEIGHT),
            Color::TRANSPARENT_WHITE,
            Some(Border::new(SECONDARY_COLOR, 1.2))
        ).shape(Shape::Round(2.0, 5)));

    }
}
