use crate::*;

#[derive(Debug, Clone)]
pub struct Trail {
    pub position: Vector2,
    progress: Animate<f32>
}

impl Trail {
    pub fn new(
        position: Vector2,
        start_time: f32,
        duration: f32,
    ) -> Self {
        Self {
            position,
            progress: Animate::new(start_time, duration, Easing::EaseOutSine, 0.0, 1.0),
        }
    }

    pub fn progress(&self, time: f32) -> f32 {
        self.progress.value(time)
    }

    pub fn complete(&self, time: f32) -> bool {
        self.progress(time) == 1.0
    }

    #[cfg(feature="graphics")]
    pub fn ripple(
        &self,
        time: f32,

        start_radius: f32,
        end_radius: f32,

        fill_color: Color,
        border: Option<Border>,
    ) -> Box<dyn TatakuRenderable> {
        let progress= self.progress(time);

        Box::new(
            Circle::new(
                self.position,
                Interpolation::lerp(start_radius, end_radius, progress),
                fill_color.alpha(Interpolation::lerp(
                    fill_color.a(), 
                    0.0,
                    progress
                )),
            ).border_maybe(border)
        )
    }
}
