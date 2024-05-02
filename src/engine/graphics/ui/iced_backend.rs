use crate::prelude::*;
use iced::advanced::graphics::backend;

pub struct IcedBackend;
impl IcedBackend {
    pub fn new() -> Self { Self }
}

impl backend::Backend for IcedBackend {
    type Primitive = Arc<dyn TatakuRenderable>;
    // fn trim_measurements(&mut self) {}
}

impl backend::Text for IcedBackend {
    const ICON_FONT: iced::Font = iced::Font::with_name("FontAwesome");
    const CHECKMARK_ICON: char = '✔';
    const ARROW_DOWN_ICON: char = '▼';

    fn default_size(&self) -> f32 { 25.0 }
    fn default_font(&self) -> iced::Font { iced::Font::DEFAULT }

    fn measure(
        &self,
        contents: &str,
        size: f32,
        _line_height: iced::advanced::text::LineHeight,
        font: iced::Font,
        _bounds: iced::Size,
        _shaping: iced::advanced::text::Shaping,
    ) -> iced::Size {
        let s = Text::new(Vector2::ZERO, size, contents, Color::WHITE, Font::from_iced(&font)).measure_text();
        iced::Size::new(s.x, s.y)
    }

    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        _line_height: iced::advanced::text::LineHeight,
        font: iced::Font,
        bounds: iced::Size,
        _shaping: iced::advanced::text::Shaping,
        point: iced::Point,
        _nearest_only: bool,
    ) -> Option<iced::advanced::text::Hit> {
        let bounds = Vector2::new(bounds.width, bounds.height);
        let mut ti = TextInput::new(Vector2::ZERO, bounds, "", contents, Font::from_iced(&font));
        ti.font_size = size;

        //TODO: not always return the thing
        Some(iced::advanced::text::Hit::CharOffset(ti.index_at_x(point.x)))
    }

    fn load_font(&mut self, _font: std::borrow::Cow<'static, [u8]>) {
        warn!("thingy wanted to load font");
        // todo!()
    }

}

impl backend::Image for IcedBackend {
    fn dimensions(&self, _handle: &iced::advanced::image::Handle) -> iced::Size<u32> {
        println!("image dimensions");
        iced::Size::new(1, 1)
        // todo!()
    }
}

