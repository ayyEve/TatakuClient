use crate::prelude::*;

const TEXT_HPADDING:f32 = 5.0;

#[derive(Default)]
pub struct CenteredTextHelper {
    text: String,

    changed_time: f32,
    draw_time: f32,
    font: DefaultFont,

    layout: Option<Arc<parley::Layout<Color>>>,
}
impl CenteredTextHelper {
    pub fn new(draw_time: f32) -> Self {
        Self {
            text: String::new(),

            draw_time,
            changed_time: 0.0,

            font: DefaultFont::Main,
            layout: None,
        }
    }

    pub fn update(&mut self, contexts: &mut TextLayoutContexts) {
        if self.layout.is_some() { return }

        let mut layout = contexts.simple_text(
            &self.text, 
            &TextStyle {
                font: self.font,
                font_size: 32.0,
                color: Color::BLACK,
                ..Default::default()
            }
        );

        layout.break_all_lines(None);
        self.layout = Some(Arc::new(layout));
    }

    pub fn set_value(&mut self, text: String, time: f32) {
        self.text = text;
        self.changed_time = time;
        self.layout = None;
    }
    pub fn _reset_timer(&mut self) {
        self.changed_time = 0.0;
    }

    pub fn draw(
        &self,
        time: f32,
        window_size: Vector2,
        list: &mut RenderableCollection,
    ) {
        let Some(layout) = self.layout.clone() 
        else { return };

        if self.changed_time > 0.0 && time - self.changed_time < self.draw_time {
            // let mut offset_text = Text::new(
            //     Vector2::ZERO, // centered later
            //     32.0,
            //     self.text.clone(),
            //     Color::BLACK,
            //     self.font
            // );
            let text_size = Vector2::new(
                layout.width(),
                layout.height(),
            );

            let text_width = text_size.x + TEXT_HPADDING;
            // center
            let rect = Bounds::new(
                Vector2::new(
                    (window_size.x - text_width) / 2.0,
                    window_size.y * 1.0 / 3.0
                ),
                Vector2::new(text_width + TEXT_HPADDING, 64.0)
            );
            let centered = Alignment::CENTER.resolve(
                &rect, 
                text_size, 
                true, 
                true,
            );

            // add
            list.push(Rectangle::new_bounds(
                rect,
                Color::WHITE.alpha(0.8),
            ));

            list.push(Transformed::new(
                Transform::default()
                    .translate(centered),
                Box::new(Text::new(layout.clone()))
            ));
        }
    }
}
