use crate::prelude::*;

const OUTER_PADDING:f32 = 5.0;
const INNER_PADDING:f32 = 8.0;
const PRIMARY_COLOR:Color = Color::WHITE;
const SECONDARY_COLOR:Color = Color::new(1.0, 1.0, 1.0, 0.1);

pub struct CurrentSongDisplay {
    current_beatmap: SyValueHelper,

    song_text: String,
    // text_size: Vector2,
}
impl CurrentSongDisplay {
    pub fn new() -> Self {
        Self {
            current_beatmap: SyValueHelper::new("map"),
            // current_song,

            song_text: String::new(),
            // text_size: Vector2::ZERO,
        }
    }

    pub fn update(&mut self, values: &ValueCollection) {
        if self.current_beatmap.check(values) {
            let a = self.current_beatmap.deref();
            let map:Result<BeatmapMeta, String> = a.try_into();
            
            self.song_text = match map {
                Ok(map) => format!("{} - {}", map.artist, map.title),
                Err(_) => String::new(),
            };
                
            //     // let text = Text::new(Vector2::ZERO, 30.0, self.song_text.clone(), PRIMARY_COLOR, Font::Main);
            //     // self.text_size = text.measure_text();
        }
    }

    pub fn view(&self, _values: &ValueCollection) -> IcedElement {
        use crate::prelude::iced_elements::*;
        
        row!(
            Space::new(Fill, Shrink),
            ContentBackground::new(
                Text::new(self.song_text.clone())
                .color(PRIMARY_COLOR)
                .size(30)
            )
            .color(Some(SECONDARY_COLOR))
            .shape(Shape::Round(5.0))
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