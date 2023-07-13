use crate::prelude::*;

const TEXT_HPADDING:f32 = 5.0;

pub struct CenteredTextHelper {
    text: String,

    changed_time: f32,
    draw_time: f32,
    font: Font,

    window_size: WindowSizeHelper,
}
impl CenteredTextHelper {
    pub async fn new(draw_time: f32) -> Self {
        Self {
            text: String::new(),

            draw_time,
            changed_time: 0.0,

            font: get_font(),
            window_size: WindowSizeHelper::new(),
        }
    }

    pub fn set_value(&mut self, text: String, time: f32) {
        self.text = text;
        self.changed_time = time;
    }
    pub fn _reset_timer(&mut self) {
        self.changed_time = 0.0;
    }

    pub fn draw(&mut self, time: f32, list: &mut RenderableCollection) {
        self.window_size.update();
        
        if self.changed_time > 0.0 && time - self.changed_time < self.draw_time {
            let mut offset_text = Text::new(
                Vector2::ZERO, // centered later
                32.0,
                self.text.clone(),
                Color::BLACK,
                self.font.clone()
            );
            
            let text_width = offset_text.measure_text().x + TEXT_HPADDING;
            // center
            let rect = Bounds::new(
                Vector2::new((self.window_size.x - text_width) / 2.0, self.window_size.y * 1.0/3.0), 
                Vector2::new(text_width + TEXT_HPADDING, 64.0)
            );
            offset_text.center_text(&rect);
            // add
            list.push(visibility_bg(rect.pos, rect.size));
            list.push(offset_text);
        }
    }
}

impl Default for CenteredTextHelper {
    fn default() -> Self {
        Self {
            font: get_font(),
            text: Default::default(),
            changed_time: Default::default(),
            draw_time: Default::default(),
            window_size: WindowSizeHelper::new(),
        }
    }
}