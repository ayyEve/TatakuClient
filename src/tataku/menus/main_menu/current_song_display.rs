use crate::prelude::*;

const OUTER_PADDING:f32 = 5.0;
const INNER_PADDING:f32 = 8.0;
const PRIMARY_COLOR:Color = Color::WHITE;
const SECONDARY_COLOR:Color = Color::new(1.0, 1.0, 1.0, 0.1);

pub struct CurrentSongDisplay {
    current_song: CurrentBeatmapHelper,
    window_size: WindowSizeHelper,

    song_text: String,
    text_size: Vector2,
}
impl CurrentSongDisplay {
    pub fn new() -> Self {
        Self {
            current_song: CurrentBeatmapHelper::new(),
            window_size: WindowSizeHelper::new(),

            song_text: String::new(),
            text_size: Vector2::ZERO,
        }
    }

    pub fn update(&mut self) {
        self.window_size.update();
        if self.current_song.update() {
            self.song_text = self.current_song.0.as_ref().map(|s|format!("{} - {}", s.artist, s.title)).unwrap_or_default();
            
            let text = Text::new(PRIMARY_COLOR, -999.0, Vector2::ZERO, 30.0, self.song_text.clone(), get_font());
            self.text_size = text.measure_text();
        }
    }

    pub fn draw(&self, list: &mut RenderableCollection) {
        let mut text = Text::new(PRIMARY_COLOR, -999.0, Vector2::ZERO, 30.0, self.song_text.clone(), get_font());

        // (window width - text width) - 2 padding, with y padding
        let pos = self.window_size.x_portion() - self.text_size.x_portion() - Vector2::with_x(OUTER_PADDING * 2.0) + Vector2::with_y(OUTER_PADDING);
        
        let rect = Rectangle::new(
            SECONDARY_COLOR, 
            -999.0,
            pos,
            self.text_size + INNER_PADDING * 2.0,
            None
        ).shape(Shape::Round(5.0, 10));
        text.center_text(&rect);
        text.pos.y -= INNER_PADDING;

        list.push(rect);
        list.push(text);
    }
}