use crate::*;
use tataku::Color;
use graphics::TatakuRenderable;

pub trait JudgementIndicator: Send + Sync {
    fn should_keep(&self, map_time: f32) -> bool;
    fn set_start_time(&mut self, time: f32);
    fn set_draw_duration(&mut self, duration: f32, settings: &Settings);
    fn draw(&self, map_time: f32, list: &mut graphics::RenderableCollection);
}

pub struct ImageJudgementIndicator {
    pub time: f32,
    draw_duration: f32,

    pub transform: graphics::Transform,
    pub image: graphics::Animation,
}

impl ImageJudgementIndicator {
    pub fn new(
        image: graphics::Animation,
        transform: graphics::Transform,
    ) -> Self {
        Self {
            time: 0.0,
            draw_duration: 0.0,

            transform,
            image,
        }
    }
}

impl JudgementIndicator for ImageJudgementIndicator {
    fn should_keep(&self, map_time: f32) -> bool {
        map_time < self.time + self.draw_duration
    }

    fn set_start_time(&mut self, time: f32) {
        self.image.set_start_time(time);
    }

    fn set_draw_duration(&mut self, mut duration: f32, settings: &Settings) {
        let count = self.image.frames.len();

        if (count > 1 && settings.common_game_settings.use_indicator_draw_duration_for_animations) || count == 1 {
            let frametime = duration / count as f32;
            self.image.frame_delay = frametime;
        } else {
            duration = self.image.frame_delay * count as f32;
        }

        self.draw_duration = duration;
    }

    fn draw(&self, map_time: f32, list: &mut tataku_graphics::RenderableCollection) {
        let fade_duration = self.draw_duration / 2.0;
        let alpha = 1.0
            - (map_time - (self.time + (self.draw_duration - fade_duration)))
            / fade_duration
        ;
        let alpha = (alpha.clamp(0.0, 1.0) * 255.0) as u8;

        let mut img = self.image.clone();

        img.update(map_time);
        if img.frames.len() == 1 {
            img.color.a = alpha;
        }
        list.push(img.with_transform(self.transform.matrix()));
    }
}

pub struct BasicJudgementIndicator {
    pub time: f32,
    draw_duration: f32,

    pub transform: graphics::Transform,
    pub color: Color,
}
impl BasicJudgementIndicator {
    /// pos, depth, radius and color are only if image is none.
    /// if image is some, it assumes the values (pos, depth, size, etc) are already set
    pub fn new(
        color: Color, 
        transform: graphics::Transform,
    ) -> Self {
        Self {
            time: 0.0,
            draw_duration: 0.0,

            transform,
            color,
        }
    }
}

impl JudgementIndicator for BasicJudgementIndicator {
    fn set_start_time(&mut self, time: f32) {
        self.time = time;
    }
    fn set_draw_duration(&mut self, duration: f32, _settings: &Settings) {
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
        let alpha = (alpha.clamp(0.0, 1.0) * 255.0) as u8;

        list.push(graphics::Circle::new(
            self.color.alpha8(alpha),
        ).with_transform(self.transform.matrix()));
    }
}
