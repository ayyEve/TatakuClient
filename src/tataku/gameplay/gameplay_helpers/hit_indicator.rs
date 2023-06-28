use crate::prelude::*;

// const POINTS_DRAW_FADE_DURATION:f32 = 200.0;

pub trait JudgementIndicator: Send + Sync{
    fn should_keep(&self, map_time: f32) -> bool;
    fn set_start_time(&mut self, time: f32);
    fn set_draw_duration(&mut self, duration: f32, settings: &Settings);
    fn draw(&mut self, map_time: f32, list: &mut RenderableCollection);
}


pub struct BasicJudgementIndicator {
    pub pos: Vector2,
    pub time: f32,

    pub radius: f32,
    pub color: Color,

    pub image: Option<Animation>,

    draw_duration: f32
}
impl BasicJudgementIndicator {
    /// pos, depth, radius and color are only if image is none.
    /// if image is some, it assumes the values (pos, depth, size, etc) are already set
    pub fn new(pos: Vector2, time: f32, radius: f32, color: Color, image: Option<Animation>) -> Self {
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
                anim.frame_delays = vec![frametime; count];
            } else {
                duration = anim.frame_delays.first().cloned().unwrap_or_default() * count as f32
            }
        }

        self.draw_duration = duration;
    }

    fn should_keep(&self, map_time: f32) -> bool {
        map_time < self.time + self.draw_duration
    }

    fn draw(&mut self, map_time: f32, list: &mut RenderableCollection) {
        let fade_duration = self.draw_duration / 2.0;
        let alpha = (1.0 - (map_time - (self.time + (self.draw_duration - fade_duration))) / fade_duration).clamp(0.0, 1.0);
        
        if let Some(img) = &self.image {
            let mut img = img.clone();
            img.update(map_time);
            if img.frames.len() == 1 {
                img.color.a = alpha;
            }
            list.push(img);
        } else {
            list.push(Circle::new(
                self.pos,
                self.radius,
                self.color.alpha(alpha),
                None
            ))
        }
    }

}
