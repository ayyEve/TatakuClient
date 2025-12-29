use crate::prelude::*;
use tataku::{
    Easing,
    Vector2,
    Animate,
    AnimationTimeline,
};

use engine::graphics;

#[derive(Default)]
pub struct Shake {
    pub animation: Option<AnimationTimeline<f32>>
}

impl Shake {
    pub fn update(&mut self, time: f32) {
        if let Some(shake) = &mut self.animation {
            shake.update(time);

            if shake.is_empty() {
                self.animation = None;
            }
        }
    }

    pub fn transform(&self, scale: f32) -> tataku::Matrix {
        if let Some(shake) = &self.animation {
            let shake = shake.last_value();

            graphics::Transform {
                pos: Vector2::new(shake * 8.0 * scale, 0.0),
                ..graphics::Transform::identity()
            }.matrix()
        } else {
            tataku::Matrix::identity()
        }
    }
}

pub fn shake_animation(
    start_time: f32,
    time_between_shakes: f32,
    shake_count: usize,
    easing: Easing,
) -> AnimationTimeline<f32> {
    let mut animations = Vec::with_capacity(shake_count);

    animations.push(Animate::new(
        start_time,
        time_between_shakes / 2.0,
        easing,
        0.0,
        1.0,
    ));

    if shake_count > 1 {
        for i in 0..shake_count-1 {
            let (start, end) = if i.is_multiple_of(2) {
                (1.0, -1.0)
            } else {
                (-1.0, 1.0)
            };

            animations.push(Animate::new(
                start_time + time_between_shakes * (i as f32 + 0.5),
                time_between_shakes,
                easing,
                start,
                end,
            ));
        }
    }

    let (start, end) = if shake_count.is_multiple_of(2) {
        (-1.0, 0.0)
    } else {
        (1.0, 0.0)
    };

    animations.push(Animate::new(
        start_time + time_between_shakes * (shake_count as f32 + 0.5),
        time_between_shakes / 2.0,
        easing,
        start,
        end,
    ));

    AnimationTimeline::new(animations, 0.0)
}
