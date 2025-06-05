use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Animate<T> {
    pub start_time: f32,
    pub duration: f32,
    pub easing: Easing,

    pub start: T,
    pub end: T,
}

impl<T> Animate<T> {
    pub fn new(
        start_time: f32,
        duration: f32,
        easing: Easing,

        start: T,
        end: T
    ) -> Self {
        Self {
            start_time,
            duration,
            easing,
            start,
            end,
        }
    }
}

impl<T> Animate<T>
where
    T: Interpolation + Clone
{
    pub fn value(&self, time: f32) -> T {
        // how long has elapsed? (minimum 0ms, max self.duration)
        let elapsed = (time - self.start_time).clamp(0.0, self.duration);

        // % for interpolation
        let mut factor = elapsed / self.duration;
        if self.duration == 0.0 {
            factor = 1.0;
        }

        self.easing.run_easing(self.start.clone(), self.end.clone(), factor)
    }
}

#[derive(Debug, Clone)]
pub struct AnimationTimeline<T> {
    /// Animations are stored in reverse time order
    /// to optimise removals.
    animations: Vec<Animate<T>>,
    last_value: T,
}

impl<T> AnimationTimeline<T> {
    pub fn new(mut animations: Vec<Animate<T>>, initial_value: T) -> Self {
        // todo: assert no overlaps
        animations.sort_by(|a, b| b.start_time.total_cmp(&a.start_time));

        Self {
            animations,
            last_value: initial_value,
        }
    }

    pub fn push(&mut self, aninmation: Animate<T>) {
        let result = self.animations.binary_search_by(|a| aninmation.start_time.total_cmp(&a.start_time));

        let index = match result {
            Ok(i) => i,
            Err(i) => i,
        };

        self.animations.insert(index, aninmation);
    }

    /// Returns animations in reverse time order.
    pub fn into_animations(self) -> Vec<Animate<T>> {
        self.animations
    }

    /// Returns true if there are no future or current animations remaining.
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

impl<T> AnimationTimeline<T>
where
    T: Interpolation + Clone
{
    pub fn last_value(&self) -> T {
        self.last_value.clone()
    }
}

impl<T> AnimationTimeline<T>
where
    T: Interpolation + Clone + PartialEq
{
    // Returns true if last_value has changed.
    pub fn update(&mut self, time: f32) -> bool {
        if self.is_empty() { return false; }

        // let position = self.animations.iter()
        //     // Get index of last animation in progress
        //     .position(|animation| time < animation.start_time + animation.duration)
        //     // If we are after all animations, snap to the last animation.
        //     .unwrap_or_default();

        let position = self.animations.iter().rev()
            .position(|animation| animation.start_time <= time)
            // If we are before all animations, snap to the first animation.
            .unwrap_or(self.animations.len() - 1);


        let animation = &self.animations[position];

        // Update value
        let temp = self.last_value.clone();
        self.last_value = animation.value(time);

        // Exclude animation if it is still in progress
        let extra = usize::from(time < animation.start_time + animation.duration);

        // Remove finished animations
        self.animations.truncate(position + extra);

        temp != self.last_value
    }
}
