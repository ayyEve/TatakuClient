use crate::prelude::*;

/// values and equations taken from https://easings.net/
#[derive(Copy, Clone, Default)]
pub enum Easing {
    #[default]
    Linear,
    // sine
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    // quadratic
    EaseInQuadratic,
    EaseOutQuadratic,
    EaseInOutQuadratic,
    // cubic
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    // quartic
    EaseInQuartic,
    EaseOutQuartic,
    EaseInOutQuartic,
    // quintic
    EaseInQuintic,
    EaseOutQuintic,
    EaseInOutQuintic,
    // exponential
    EaseInExponential,
    EaseOutExponential,
    EaseInOutExponential,
    // circular
    EaseInCircular,
    EaseOutCircular,
    EaseInOutCircular,
    // back
    EaseInBack(f32, f32),
    EaseOutBack(f32, f32),
    EaseInOutBack(f32, f32),
}
impl Easing {
    pub fn run_easing<I:Interpolation>(self, start:I, end:I, amount: f32) -> I {
        match self {
            Easing::Linear => Interpolation::lerp(start, end, amount),

            Easing::EaseInSine => Interpolation::easein_sine(start, end, amount),
            Easing::EaseOutSine => Interpolation::easeout_sine(start, end, amount),
            Easing::EaseInOutSine => Interpolation::easeinout_sine(start, end, amount),

            Easing::EaseInQuadratic => Interpolation::easein_quadratic(start, end, amount),
            Easing::EaseOutQuadratic => Interpolation::easeout_quadratic(start, end, amount),
            Easing::EaseInOutQuadratic => Interpolation::easeinout_quadratic(start, end, amount),

            Easing::EaseInCubic => Interpolation::easein_cubic(start, end, amount),
            Easing::EaseOutCubic => Interpolation::easeout_cubic(start, end, amount),
            Easing::EaseInOutCubic => Interpolation::easeinout_cubic(start, end, amount),

            Easing::EaseInQuartic => Interpolation::easein_quartic(start, end, amount),
            Easing::EaseOutQuartic => Interpolation::easeout_quartic(start, end, amount),
            Easing::EaseInOutQuartic => Interpolation::easeinout_quartic(start, end, amount),

            Easing::EaseInQuintic => Interpolation::easein_quintic(start, end, amount),
            Easing::EaseOutQuintic => Interpolation::easeout_quintic(start, end, amount),
            Easing::EaseInOutQuintic => Interpolation::easeinout_quintic(start, end, amount),

            Easing::EaseInExponential => Interpolation::easein_exponential(start, end, amount),
            Easing::EaseOutExponential => Interpolation::easeout_exponential(start, end, amount),
            Easing::EaseInOutExponential => Interpolation::easeinout_exponential(start, end, amount),

            Easing::EaseInCircular => Interpolation::easein_circular(start, end, amount),
            Easing::EaseOutCircular => Interpolation::easeout_circular(start, end, amount),
            Easing::EaseInOutCircular => Interpolation::easeinout_circular(start, end, amount),

            Easing::EaseInBack(c1, c2) => Interpolation::easein_back(start, end, amount, c1, c2),
            Easing::EaseOutBack(c1, c2) => Interpolation::easeout_back(start, end, amount, c1, c2),
            Easing::EaseInOutBack(c1, c2) => Interpolation::easeinout_back(start, end, amount, c1, c2),
        }
    }
}
