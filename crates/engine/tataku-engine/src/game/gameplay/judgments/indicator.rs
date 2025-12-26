use crate::*;
use tataku::Vector2;
use tataku::Color;

pub trait JudgementIndicator: Send + Sync {
    fn should_keep(&self, map_time: f32) -> bool;
    fn set_start_time(&mut self, time: f32);
    fn set_draw_duration(&mut self, duration: f32, settings: &Settings);
    fn draw(&self, map_time: f32, list: &mut graphics::RenderableCollection);
}

pub struct BasicJudgementIndicator {
    pub pos: Vector2,
    pub time: f32,

    pub radius: f32,
    pub color: Color,

    pub image: Option<graphics::Animation>,

    draw_duration: f32
}
impl BasicJudgementIndicator {
    /// pos, depth, radius and color are only if image is none.
    /// if image is some, it assumes the values (pos, depth, size, etc) are already set
    pub fn new(
        pos: Vector2, 
        time: f32, 
        radius: f32, 
        color: Color, 
        image: Option<graphics::Animation>
    ) -> Self {
        Self {
            pos,
            time,
            radius,
            color,
            image,
            draw_duration: 0.0
        }
    }
}

impl JudgementIndicator for BasicJudgementIndicator {
    fn set_start_time(&mut self, time: f32) {
        if let Some(anim) = &mut self.image {
            anim.set_start_time(time);
        }
    }
    fn set_draw_duration(&mut self, mut duration: f32, settings: &Settings) {
        if let Some(anim) = &mut self.image {
            let count = anim.frames.len();
            
            if (count > 1 && settings.common_game_settings.use_indicator_draw_duration_for_animations) || count == 1 {
                let frametime = duration / count as f32;
                anim.frame_delay = frametime;
            } else {
                duration = anim.frame_delay * count as f32;
            }
        }

        self.draw_duration = duration;
    }

    fn should_keep(&self, map_time: f32) -> bool {
        map_time < self.time + self.draw_duration
    }

    fn draw(&self, map_time: f32, list: &mut graphics::RenderableCollection) {
        let fade_duration = self.draw_duration / 2.0;
        let alpha = 1.0 
            - (map_time - (self.time + (self.draw_duration - fade_duration))) 
            / fade_duration
        ;
        let alpha = tataku::ColorField::new_f32(alpha);
        
        if let Some(mut img) = self.image.clone() {
            img.update(map_time);
            if img.frames.len() == 1 {
                img.color.a = alpha;
            }
            list.push(img);
        } else {
            list.push(graphics::Circle::new(
                self.pos,
                self.radius,
                self.color.with_alpha(alpha),
            ));
        }
    }
}
