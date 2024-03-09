use crate::prelude::*;

const OUTER_PADDING:f32 = 5.0;
const INNER_PADDING:f32 = 8.0;
const PRIMARY_COLOR:Color = Color::WHITE;
const SECONDARY_COLOR:Color = Color::new(1.0, 1.0, 1.0, 0.1);

pub struct CurrentSongDisplay {
    current_song: CurrentBeatmapHelper,
    // window_size: WindowSizeHelper,

    song_text: String,
    // text_size: Vector2,
}
impl CurrentSongDisplay {
    pub fn new() -> Self {
        let current_song = CurrentBeatmapHelper::new();
        let song_text = Self::get_text(&current_song);
        Self {
            current_song,
            // window_size: WindowSizeHelper::new(),

            song_text,
            // text_size: Vector2::ZERO,
        }
    }
    fn get_text(current_song: &CurrentBeatmapHelper) -> String {
        current_song.0.as_ref().map(|s|format!("{} - {}", s.artist, s.title)).unwrap_or_default()
    }

    pub fn update(&mut self) {
        // self.window_size.update();
        if self.current_song.update() {
            self.song_text = Self::get_text(&self.current_song);
            
            // let text = Text::new(Vector2::ZERO, 30.0, self.song_text.clone(), PRIMARY_COLOR, Font::Main);
            // self.text_size = text.measure_text();
        }
    }

    pub fn view(&self) -> IcedElement {
        use crate::prelude::iced_elements::*;
        use crate::prelude::Rectangle;
        
        row!(
            Space::new(Fill, Shrink),
            ContentBackground::new(
                Text::new(self.song_text.clone())
                .color(PRIMARY_COLOR)
                .size(30)
            )
            .rect(Some(Rectangle::style_only(SECONDARY_COLOR, None, Shape::Round(5.0))))
            .padding(INNER_PADDING)
            ;
            width = Fill,
            height = Shrink,
            padding = OUTER_PADDING
        )
    }

    // pub fn draw(&self, list: &mut RenderableCollection) {
    //     let mut text = Text::new(Vector2::ZERO, 30.0, self.song_text.clone(), PRIMARY_COLOR, Font::Main);

    //     // (window width - text width) - 2 padding, with y padding
    //     let pos = self.window_size.x_portion() - self.text_size.x_portion() - Vector2::with_x(OUTER_PADDING * 2.0) + Vector2::with_y(OUTER_PADDING);
        
    //     let rect = Rectangle::new(
    //         pos,
    //         self.text_size + INNER_PADDING * 2.0,
    //         SECONDARY_COLOR, 
    //         None
    //     ).shape(Shape::Round(5.0));
    //     text.center_text(&rect);
    //     text.pos.y -= INNER_PADDING;

    //     list.push(rect);
    //     list.push(text);
    // }
}

impl core::fmt::Debug for CurrentSongDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CurrentSongDisplay")
    }
}

impl Clone for CurrentSongDisplay {
    fn clone(&self) -> Self {
        Self::new()
    }
}