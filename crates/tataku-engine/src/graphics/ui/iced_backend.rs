use crate::prelude::*;

pub const DEFAULT_FONT_SIZE: f32 = 25.0;

#[derive(Default)]
pub struct IcedRenderer {
    renderables: Vec<Arc<dyn TatakuRenderable>>,
    object_stack: Vec<TransformGroup>,
    pub ui_scale: f32,
}
impl IcedRenderer {
    pub fn new() -> Self {
        Self::default()
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
        // let mut a = TransformGroup::new(bounds.position().into());
        let mut a = TransformGroup::new(Vector2::ZERO);

        let bounds:Bounds = bounds.into();
        a.set_scissor(Some(bounds.into_scissor()));
        self.object_stack.push(a);
    }

    fn end_layer(&mut self) { self.pop_last() }
    fn end_transformation(&mut self) { self.pop_last() }

    fn start_transformation(&mut self, transformation: iced::Transformation) {
        let trans = transformation.translation().into();
        let a = TransformGroup::new(trans)
            .scale(Vector2::ONE * transformation.scale_factor());
        self.object_stack.push(a);
    }


    fn fill_quad(&mut self, quad: iced_core::renderer::Quad, background: impl Into<iced::Background>) {
        let background:iced::Background = background.into();
        let color = match background {
            iced::Background::Color(color) => color.into(),
            iced::Background::Gradient(_g) => Color::ALIEN_GREEN,
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
    type Font = Font;
    type Paragraph = IcedParagraph;
    type Editor = IcedEditor;

    const ICON_FONT: Self::Font = Font::FontAwesome;
    const CHECKMARK_ICON: char = '✔';
    const ARROW_DOWN_ICON: char = '▼';

    fn default_font(&self) -> Self::Font { Font::Main }
    fn default_size(&self) -> iced::Pixels { iced::Pixels(DEFAULT_FONT_SIZE) }

    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: iced::Point,
        color: iced::Color,
        _clip_bounds: iced::Rectangle,
    ) {
        let height = text.line_height.to_absolute(text.font_size).0;
        let mut pos:Vector2 = position.into();

        for line in &text.lines {
            let mut out_text = Text::new(
                pos,
                text.font_size.0,
                line,
                color.into(),
                text.font
            );
            // out_text.set_scissor(Some([clip_bounds.x, clip_bounds.y, clip_bounds.width, clip_bounds.height]));

            match text.alignment.0 {
                iced::alignment::Horizontal::Left => {}
                iced::alignment::Horizontal::Center => out_text.pos.x += text.bounds.width - out_text.measure_text().x / 2.0,
                iced::alignment::Horizontal::Right => out_text.pos.x += text.bounds.width - out_text.measure_text().x,
            }
            match text.alignment.1 {
                iced::alignment::Vertical::Bottom => out_text.pos.y -= height,
                iced::alignment::Vertical::Center => out_text.pos.y -= height / 2.0,
                iced::alignment::Vertical::Top => {}
            }
            
            self.add_renderable(Arc::new(out_text));
            pos.y += height;
        }
        
    }

    /// i'm not convinced this is actuall used by my code
    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: iced::Point,
        _color: iced::Color,
        _clip_bounds: iced::Rectangle,
    ) { 
        todo!("not using multiline editors yet (iced::widget::TextEditor")
    }

    fn fill_text(
        &mut self,
        text: iced_core::Text<String, Self::Font>,
        position: iced::Point,
        color: iced::Color,
        _clip_bounds: iced::Rectangle,
    ) {
        let height = text.line_height.to_absolute(text.size).0;
        
        let mut out_text = Text::new(
            position.into(),
            text.size.0,
            text.content,
            color.into(),
            text.font
        );
        // out_text.set_scissor(Some([clip_bounds.x, clip_bounds.y, clip_bounds.width, clip_bounds.height]));

        match text.vertical_alignment {
            iced::alignment::Vertical::Bottom => out_text.pos.y -= height,
            iced::alignment::Vertical::Center => out_text.pos.y -= height / 2.0,
            iced::alignment::Vertical::Top => {}
        }
        match text.horizontal_alignment {
            iced::alignment::Horizontal::Left => {}
            iced::alignment::Horizontal::Center => out_text.pos.x += text.bounds.width - out_text.measure_text().x / 2.0,
            iced::alignment::Horizontal::Right => out_text.pos.x += text.bounds.width - out_text.measure_text().x,
        }
        
        self.add_renderable(Arc::new(out_text))
    }
}

// impl backend::Image for IcedBackend {
//     fn dimensions(&self, _handle: &iced::advanced::image::Handle) -> iced::Size<u32> {
//         println!("image dimensions");
//         iced::Size::new(1, 1)
//         // todo!()
//     }
// }

pub struct IcedParagraph {
    text_raw: String,
    lines: Vec<String>,
    /// pre-calculated text size
    text_size: Vector2,

    font: Font,
    font_size: iced::Pixels,
    line_height: iced_core::text::LineHeight,
    bounds: iced::Size<f32>,
    alignment: (iced::alignment::Horizontal, iced::alignment::Vertical),
    // shaping: iced_core::text::Shaping,
}
impl iced::advanced::text::Paragraph for IcedParagraph {
    type Font = Font;

    fn with_text(text: iced_core::Text<&str, Self::Font>) -> Self {
        let size = Text::measure_text_raw(
            &[text.font], 
            text.size.0, 
            text.content, 
            Vector2::ONE, 
            text.line_height.to_absolute(text.size).0 - text.size.0
        );

        Self {
            text_raw: text.content.to_owned(),
            lines: text.content.lines().map(|s| s.to_string()).collect(),
            text_size: size,

            bounds: text.bounds,
            font_size: text.size,
            line_height: text.line_height,
            font: text.font,
            alignment: (text.horizontal_alignment, text.vertical_alignment),
            // shaping: text.shaping,
        }
    }

    fn resize(&mut self, new_bounds: iced::Size) {
        self.bounds = new_bounds;
    }

    fn compare(&self, text: iced_core::Text<(), Self::Font>) -> iced_core::text::Difference {
        if (text.horizontal_alignment, text.vertical_alignment) != self.alignment
        || text.font != self.font
        || text.size != self.font_size
        || text.line_height != self.line_height
        // || text.content != self.text_raw
        {
            return iced_core::text::Difference::Shape;
        }

        if self.bounds != text.bounds {
            return iced_core::text::Difference::Bounds;
        }

        iced_core::text::Difference::None
    }

    fn horizontal_alignment(&self) -> iced::alignment::Horizontal {
        self.alignment.0
    }

    fn vertical_alignment(&self) -> iced::alignment::Vertical {
        self.alignment.1
    }

    fn min_bounds(&self) -> iced::Size {
        iced::Size::new(
            self.text_size.x,
            self.text_size.y
        )
    }

    fn hit_test(&self, point: iced::Point) -> Option<iced_core::text::Hit> {
        // if !iced::Rectangle::new(iced::Point::new(0.0, 0.0), self.bounds).contains(point) {
        //     return None;
        // }

        let (font_size, text_scale) = Text::get_font_size_scaled(self.font_size.0);

        use iced_core::text::Hit::CharOffset;
        //TODO: eventually we will need to care about y coords here as well.

        if point.x > self.text_size.x { return Some(CharOffset(self.text_raw.len()-1)) }
        let pos = Vector2::ZERO;

        if pos.x > point.x { return Some(CharOffset(self.lines.len())); }


        // cumulative width
        let mut width = 0.0;

        for (counter, char) in self.text_raw.char_indices() {
            // get the font character
            let Some(c) = self.font.get_character(font_size, char) else { continue };

            let a = c.advance_width() * text_scale / 2.0;

            width += a;
            
            if point.x < width { return Some(CharOffset(counter)); }

            width += a;
        }

        Some(CharOffset(self.lines.len()))
    }

    fn grapheme_position(&self, line: usize, index: usize) -> Option<iced::Point> {
        let text = self.lines.get(line)?;
        
        // cumulative width
        let mut width = 0.0;
        let (font_size, text_scale) = Text::get_font_size_scaled(self.font_size.0);

        for (counter, char) in text.char_indices() {
            if counter == index { break }

            // get the font character
            let Some(c) = self.font.get_character(font_size, char) else { continue };
            
            width += c.advance_width() * text_scale;
        }

        Some(iced::Point::new(
            width, 
            self.line_height.to_absolute(iced::Pixels(font_size)).0 * text_scale * (line as f32)
        ))
    }
    
    fn with_spans<Link>(
        text: iced_core::Text<&[iced_core::text::Span<'_, Link, Self::Font>], Self::Font>,
    ) -> Self {
        let a = text.content
            .iter()
            .map(|a| a.text.clone().into_owned())
            .collect::<Vec<_>>()
            .join("");

        Self::with_text(iced_core::Text {
            content: &a,
            bounds: text.bounds,
            size: text.size,
            line_height: text.line_height,
            font: text.font,
            horizontal_alignment: text.horizontal_alignment,
            vertical_alignment: text.vertical_alignment,
            shaping: text.shaping,
            wrapping: text.wrapping,
        })
    }
    
    // TODO!
    fn hit_span(&self, _point: iced::Point) -> Option<usize> {
        None
    }
    
    // TODO!
    fn span_bounds(&self, _index: usize) -> Vec<iced::Rectangle> {
        vec![iced::Rectangle::new(iced::Point::default(), self.bounds)]
    }
}
impl Default for IcedParagraph {
    fn default() -> Self {
        Self {
            text_raw: String::new(),
            lines: Vec::new(),
            text_size: Vector2::ZERO,
        
            font: Font::Main,
            font_size: iced::Pixels(25.0),
            line_height: iced_core::text::LineHeight::default(),
            bounds: iced::Size::default(),
            alignment: (iced::alignment::Horizontal::Left, iced::alignment::Vertical::Top),
            // shaping: iced_core::text::Shaping::Basic,
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

    fn cursor_position(&self) -> (usize, usize) { (0, 0) }

    fn selection(&self) -> Option<String> { None }

    fn line(&self, index: usize) -> Option<&str> {
        if index == 0 {
            return Some(&self.text)
        }
        None
    }

    fn line_count(&self) -> usize { 1 }

    fn perform(&mut self, _action: iced_core::text::editor::Action) { }

    fn bounds(&self) -> iced::Size { self.bounds }

    fn min_bounds(&self) -> iced::Size { self.bounds }
 
    fn update(
        &mut self,
        new_bounds: iced::Size,
        _new_font: Self::Font,
        _new_size: iced::Pixels,
        _new_line_height: iced_core::text::LineHeight,
        _new_wrapping: iced_core::text::Wrapping,
        _new_highlighter: &mut impl iced_core::text::Highlighter,
    ) {
        self.bounds = new_bounds;
    }

    fn highlight<H: iced_core::text::Highlighter>(
        &mut self,
        _font: Self::Font,
        _highlighter: &mut H,
        _format_highlight: impl Fn(&H::Highlight) -> iced_core::text::highlighter::Format<Self::Font>,
    ) { }
    
    fn is_empty(&self) -> bool {
        true
    }
}