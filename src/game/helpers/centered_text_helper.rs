use crate::prelude::*;

const TEXT_HPADDING:f64 = 5.0;

pub struct CenteredTextHelper<V:Display> {
    label: String,
    pub value: V,

    pub depth: f64,
    changed_time: f32,
    draw_time: f32,
    font: Font,
}
impl<V:Display> CenteredTextHelper<V>{
    pub fn new(label:&str, initial_value: V, draw_time: f32, depth: f64, font: Font) -> Self {
        Self {
            label: label.to_owned(),
            value: initial_value,

            depth,
            draw_time,
            changed_time: 0.0,

            font
        }
    }

    pub fn set_value(&mut self, new_value:V, time: f32) {
        self.value = new_value;
        self.changed_time = time;
    }
    pub fn _reset_timer(&mut self) {
        self.changed_time = 0.0;
    }

    pub fn draw(&mut self, time: f32, list: &mut Vec<Box<dyn Renderable>>) {
        let window_size = Settings::window_size();

        if self.changed_time > 0.0 && time - self.changed_time < self.draw_time {
            let mut offset_text = Text::new(
                Color::BLACK,
                self.depth,
                Vector2::zero(), // centered anyways
                32,
                format!("{}: {}", self.label, self.value),
                self.font.clone()
            );
            
            let text_width = offset_text.measure_text().x + TEXT_HPADDING;
            // center
            let rect = Rectangle::bounds_only(
                Vector2::new((window_size.x - text_width) / 2.0, window_size.y * 1.0/3.0), 
                Vector2::new( text_width + TEXT_HPADDING, 64.0)
            );
            offset_text.center_text(rect);
            // add
            list.push(visibility_bg(rect.current_pos, rect.size, self.depth + 10.0));
            list.push(Box::new(offset_text));
        }
    }
}
impl<V:Display + Default> Default for CenteredTextHelper<V> {
    fn default() -> Self {
        Self {
            font: get_font(),
            label: Default::default(),
            value: Default::default(),
            depth: Default::default(),
            changed_time: Default::default(),
            draw_time: Default::default(),
        }
    }
}