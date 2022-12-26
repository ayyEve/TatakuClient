use crate::prelude::*;

const POINTS_DRAW_FADE_DURATION:f32 = 60.0;

pub trait JudgementIndicator: Send + Sync{
    fn should_keep(&self, map_time: f32) -> bool;
    fn set_draw_duration(&mut self, duration: f32);
    fn draw(&mut self, map_time: f32, list: &mut RenderableCollection);
}


pub struct BasicJudgementIndicator {
    pub pos: Vector2,
    pub time: f32,
    pub depth: f64,

    pub radius: f64,
    pub color: Color,

    pub image: Option<Image>,

    draw_duration: f32
}
impl BasicJudgementIndicator {
    /// pos, depth, radius and color are only if image is none.
    /// if image is some, it assumes the values (pos, depth, size, etc) are already set
    pub fn new(pos: Vector2, time: f32, depth: f64, radius: f64, color: Color, image: Option<Image>) -> Self {
        Self {
            pos,
            time,
            depth,
            radius,
            color,
            image,
            draw_duration: 0.0
        }
    }
}

impl JudgementIndicator for BasicJudgementIndicator {
    fn set_draw_duration(&mut self, duration: f32) {
        self.draw_duration = duration
    }

    fn should_keep(&self, map_time: f32) -> bool {
        map_time < self.time + self.draw_duration
    }

    fn draw(&mut self, map_time: f32, list: &mut RenderableCollection) {
        let alpha = (1.0 - (map_time - (self.time + (self.draw_duration - POINTS_DRAW_FADE_DURATION))) / POINTS_DRAW_FADE_DURATION).clamp(0.0, 1.0);
        
        if let Some(img) = &self.image {
            let mut img = img.clone();
            img.color.a = alpha;
            list.push(img);
        } else {
            list.push(Circle::new(
                self.color.alpha(alpha),
                self.depth,
                self.pos,
                self.radius,
                None
            ))
        }
    }

}
