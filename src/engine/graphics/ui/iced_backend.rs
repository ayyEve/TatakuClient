use crate::prelude::*;
// use iced::advanced::graphics::backend;


pub struct IcedRenderer {
    renderables: Vec<Arc<dyn TatakuRenderable>>,
    object_stack: Vec<TransformGroup>
}
impl IcedRenderer {
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            object_stack: Vec::new(),
        }
    }

    pub fn finish(&mut self) -> TransformGroup {
        let mut group = TransformGroup::new(Vector2::ZERO);
        group.items = self.renderables.clone();
        group
    }

    pub fn add_renderable(&mut self, renderable: Arc<dyn TatakuRenderable>) {
        if let Some(last) = self.object_stack.last_mut() {
            last.push_arced(renderable);
        } else {
            self.renderables.push(renderable);
        }
    }

    fn pop_last(&mut self) {
        let Some(a) = self.object_stack.pop() else { return };
        self.add_renderable(Arc::new(a));
    }
}

impl iced::advanced::Renderer for IcedRenderer {
    fn start_layer(&mut self, bounds: iced::Rectangle) {
        let mut a = TransformGroup::new(bounds.position().into());
        a.set_scissor(Some([ bounds.x, bounds.y, bounds.width, bounds.height ]));
        self.object_stack.push(a);
    }

    fn end_layer(&mut self) { self.pop_last() }
    fn end_transformation(&mut self) { self.pop_last() }

    fn start_transformation(&mut self, transformation: iced::Transformation) {
        let trans = transformation.translation();
        let a = TransformGroup::new(Vector2::new(trans.x, trans.y))
            .scale(Vector2::ONE * transformation.scale_factor());
        self.object_stack.push(a);
    }


    fn fill_quad(&mut self, quad: iced_core::renderer::Quad, background: impl Into<iced::Background>) {
        let background:iced::Background = background.into();
        let color = match background {
            iced::Background::Color(color) => color.into(),
            iced::Background::Gradient(g) => Color::ALIEN_GREEN,
        };

        self.add_renderable(Arc::new(Rectangle::new(
            Vector2::new(quad.bounds.x, quad.bounds.y),
            Vector2::new(quad.bounds.width, quad.bounds.height),
            color,
            Some(Border::new(quad.border.color.into(), quad.border.width))
        ).shape(Shape::RoundSep(quad.border.radius.into()))))
    }

    fn clear(&mut self) {
        self.renderables.clear();
        self.object_stack.clear();
    }
}

impl iced::advanced::text::Renderer for IcedRenderer {
    type Font = crate::prelude::Font;
    type Paragraph = IcedParagraph;
    type Editor = IcedEditor;

    const ICON_FONT: Self::Font = Font::FontAwesome;
    const CHECKMARK_ICON: char = '✔';
    const ARROW_DOWN_ICON: char = '▼';

    fn default_font(&self) -> Self::Font { Font::Main }
    fn default_size(&self) -> iced::Pixels { iced::Pixels(25.0) }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: iced::Point,
        color: iced::Color,
        clip_bounds: iced::Rectangle,
    ) {

    }

    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: iced::Point,
        color: iced::Color,
        clip_bounds: iced::Rectangle,
    ) {

    }

    fn fill_text(
        &mut self,
        text: iced_core::Text<String, Self::Font>,
        position: iced::Point,
        color: iced::Color,
        clip_bounds: iced::Rectangle,
    ) {

    }
}


// impl backend::Text for IcedBackend {
//     const ICON_FONT: iced::Font = iced::Font::with_name("FontAwesome");
//     const CHECKMARK_ICON: char = '✔';
//     const ARROW_DOWN_ICON: char = '▼';

//     fn default_size(&self) -> f32 { 25.0 }
//     fn default_font(&self) -> iced::Font { iced::Font::DEFAULT }

//     fn measure(
//         &self,
//         contents: &str,
//         size: f32,
//         _line_height: iced::advanced::text::LineHeight,
//         font: iced::Font,
//         _bounds: iced::Size,
//         _shaping: iced::advanced::text::Shaping,
//     ) -> iced::Size {
//         let s = Text::new(Vector2::ZERO, size, contents, Color::WHITE, Font::from_iced(&font)).measure_text();
//         iced::Size::new(s.x, s.y)
//     }

//     fn hit_test(
//         &self,
//         contents: &str,
//         size: f32,
//         _line_height: iced::advanced::text::LineHeight,
//         font: iced::Font,
//         bounds: iced::Size,
//         _shaping: iced::advanced::text::Shaping,
//         point: iced::Point,
//         _nearest_only: bool,
//     ) -> Option<iced::advanced::text::Hit> {
//         let bounds = Vector2::new(bounds.width, bounds.height);
//         let mut ti = TextInput::new(Vector2::ZERO, bounds, "", contents, Font::from_iced(&font));
//         ti.font_size = size;

//         //TODO: not always return the thing
//         Some(iced::advanced::text::Hit::CharOffset(ti.index_at_x(point.x)))
//     }

//     fn load_font(&mut self, _font: std::borrow::Cow<'static, [u8]>) {
//         warn!("thingy wanted to load font");
//         // todo!()
//     }

// }

// impl backend::Image for IcedBackend {
//     fn dimensions(&self, _handle: &iced::advanced::image::Handle) -> iced::Size<u32> {
//         println!("image dimensions");
//         iced::Size::new(1, 1)
//         // todo!()
//     }
// }


pub struct IcedParagraph {
    text: iced_core::Text<String, Font>,
}
impl iced::advanced::text::Paragraph for IcedParagraph {
    type Font = Font;

    fn with_text(text: iced_core::Text<&str, Self::Font>) -> Self {
        Self {
            text: iced_core::Text {
                content: text.content.to_owned(),
                bounds: text.bounds,
                size: text.size,
                line_height: text.line_height,
                font: text.font,
                horizontal_alignment: text.horizontal_alignment,
                vertical_alignment: text.vertical_alignment,
                shaping: text.shaping,
            }
        }
    }

    fn resize(&mut self, new_bounds: iced::Size) {

    }

    fn compare(&self, text: iced_core::Text<&str, Self::Font>) -> iced_core::text::Difference {
        iced_core::text::Difference::None
    }

    fn horizontal_alignment(&self) -> iced::alignment::Horizontal {
        self.text.horizontal_alignment
    }

    fn vertical_alignment(&self) -> iced::alignment::Vertical {
        self.text.vertical_alignment
    }

    fn min_bounds(&self) -> iced::Size {
        self.text.bounds
    }

    fn hit_test(&self, point: iced::Point) -> Option<iced_core::text::Hit> {
        None
    }

    fn grapheme_position(&self, line: usize, index: usize) -> Option<iced::Point> {
        None
    }
}

impl Default for IcedParagraph {
    fn default() -> Self {
        Self {
            text: iced_core::Text {
                content: String::new(),
                bounds: iced::Size::new(0.0, 0.0),
                size: iced::Pixels(25.0),
                line_height: iced_core::text::LineHeight::Relative(0.0),
                font: Font::Main,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Top,
                shaping: iced_core::text::Shaping::Basic,
            }
        }
    }
}


#[derive(Default)]
pub struct IcedEditor {
    text: String,
    bounds: iced::Size
}

impl iced::advanced::text::Editor for IcedEditor {
    type Font = Font;

    fn with_text(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            bounds: Default::default(),
        }
    }

    fn cursor(&self) -> iced_core::text::editor::Cursor {
        iced_core::text::editor::Cursor::Caret(iced::Point::new(0.0, 0.0))
    }

    fn cursor_position(&self) -> (usize, usize) {
        (0, 0)
    }

    fn selection(&self) -> Option<String> {
        None
    }

    fn line(&self, index: usize) -> Option<&str> {
        None
    }

    fn line_count(&self) -> usize {
        1
    }

    fn perform(&mut self, action: iced_core::text::editor::Action) {
        
    }

    fn bounds(&self) -> iced::Size {
        self.bounds
    }

    fn min_bounds(&self) -> iced::Size {
        self.bounds
    }

    fn update(
        &mut self,
        new_bounds: iced::Size,
        new_font: Self::Font,
        new_size: iced::Pixels,
        new_line_height: iced_core::text::LineHeight,
        new_highlighter: &mut impl iced_core::text::Highlighter,
    ) {
        self.bounds = new_bounds;
    }

    fn highlight<H: iced_core::text::Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> iced_core::text::highlighter::Format<Self::Font>,
    ) {
        
    }
}