use crate::prelude::PI;

macro_rules! check_bounds {
    ($current:expr, $target:expr, $amount:expr) => {
        if $amount == 0.0 {
            return $current
        }
        if $amount == 1.0 {
            return $target
        }
    };
}


pub trait Interpolation {
    fn lerp(current: Self, target: Self, amount: f32) -> Self;

    // helpers since many of the easing fns are just different powers
    fn ease_in_exp(current: Self, target: Self, amount: f32, pow: i32) -> Self;
    fn ease_out_exp(current: Self, target: Self, amount: f32, pow: i32) -> Self;
    fn ease_inout_exp(current: Self, target: Self, amount: f32, pow: i32) -> Self;

    // sine
    fn easein_sine(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_sine(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_sine(current:Self, target: Self, amount: f32) -> Self;

    // quadratic
    fn easein_quadratic(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_quadratic(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_quadratic(current:Self, target: Self, amount: f32) -> Self;

    // cubic
    fn easein_cubic(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_cubic(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_cubic(current:Self, target: Self, amount: f32) -> Self;

    // quartic
    fn easein_quartic(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_quartic(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_quartic(current:Self, target: Self, amount: f32) -> Self;

    // quintic
    fn easein_quintic(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_quintic(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_quintic(current:Self, target: Self, amount: f32) -> Self;

    // exponential
    fn easein_exponential(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_exponential(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_exponential(current:Self, target: Self, amount: f32) -> Self;

    // circular
    fn easein_circular(current:Self, target: Self, amount: f32) -> Self;
    fn easeout_circular(current:Self, target: Self, amount: f32) -> Self;
    fn easeinout_circular(current:Self, target: Self, amount: f32) -> Self;

    // back
    // todo! come up with better names than c1 and c3
    fn easein_back(current:Self, target: Self, amount: f32, c1:f32, c3: f32) -> Self;
    fn easeout_back(current:Self, target: Self, amount: f32, c1:f32, c3: f32) -> Self;
    fn easeinout_back(current:Self, target: Self, amount: f32, c1:f32, c3: f32) -> Self;

    // skipping elastic and bounce bc they kinda suck
}
impl<T> Interpolation for T where T: Copy + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + std::ops::Mul<f32, Output=T> {
    fn lerp(current:T, target:T,  amount:f32) -> T {
        current + (target - current) * amount
    }

    // helpers
    fn ease_in_exp(current:T, target:T, amount:f32, pow:i32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, amount.powi(pow))
    }
    fn ease_out_exp(current:T, target:T, amount:f32, pow:i32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - (1.0 - amount).powi(pow))
    }
    fn ease_inout_exp(current:T, target:T, amount:f32, pow:i32) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount < 0.5 {
            2.0f32.powi(pow - 1) * amount.powi(pow)
        } else {
            1.0 - (-2.0 * amount + 2.0).powi(pow) / 2.0
        };
        Self::lerp(current, target, amount)
    }

    // sine
    fn easein_sine(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - ((amount * PI) / 2.0).cos())
    }
    fn easeout_sine(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, ((amount * PI) / 2.0).sin())
    }
    fn easeinout_sine(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, -((amount * PI).cos() - 1.0) / 2.0)
    }

    // quad
    fn easein_quadratic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 2)
    }
    fn easeout_quadratic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 2)
    }
    fn easeinout_quadratic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 2)
    }

    // cubic
    fn easein_cubic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 3)
    }
    fn easeout_cubic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 3)
    }
    fn easeinout_cubic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 3)
    }

    // quart
    fn easein_quartic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 4)
    }
    fn easeout_quartic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 4)
    }
    fn easeinout_quartic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 4)
    }

    // quint
    fn easein_quintic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 5)
    }
    fn easeout_quintic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 5)
    }
    fn easeinout_quintic(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 5)
    }

    // expo
    fn easein_exponential(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount == 0.0 {0.0} else {
            2f32.powf(amount * 10.0  - 10.0)
        };
        Self::lerp(current, target, amount)
    }
    fn easeout_exponential(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount == 1.0 {1.0} else {
            1.0 - 2f32.powf(amount * -10.0)
        };
        Self::lerp(current, target, amount)
    }
    fn easeinout_exponential(current:T, target:T, amount:f32) -> T {
        check_bounds!(current, target, amount);
        let amount =
            if amount == 0.0 {0.0}
            else if amount == 1.0 {1.0}
            else if amount < 0.5 {
                2f32.powf(20.0 * amount - 10.0) / 2.0
            } else {
                (2.0 - 2f32.powf(-20.0 * amount + 10.0)) / 2.0
            };
        Self::lerp(current, target, amount)
    }

    // circular
    fn easein_circular(current:T, target: T, amount: f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - (1.0 - amount.powi(2)).sqrt())
    }
    fn easeout_circular(current:T, target: T, amount: f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - ((amount - 1.0).powi(2)).sqrt())
    }
    fn easeinout_circular(current:T, target: T, amount: f32) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount < 0.5 {
            (1.0 - (1.0 - (2.0 * amount).powi(2)).sqrt()) / 2.0
        } else {
            ((1.0 - (-2.0 * amount + 2.0).powi(2)).sqrt() + 1.0) / 2.0
        };
        Self::lerp(current, target, amount)
    }

    // back
    fn easein_back(current:T, target: T, amount: f32, c1:f32, c3:f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, c3 * amount.powi(3) - c1 * amount.powi(2))
    }
    fn easeout_back(current:T, target: T, amount: f32, c1:f32, c3: f32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 + c3 * (amount - 1.0).powi(3) - c1 * (amount - 1.0).powi(2))
    }
    fn easeinout_back(current:T, target: T, amount: f32, _c1:f32, c2: f32) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount < 0.5 {
            (
                (2.0 * amount).powi(2)
                * (
                    (c2 + 1.0)
                    * 2.0
                    * amount
                    - c2
                )
            ) / 2.0
        } else {
            (
                (2.0 * amount - 2.0).powi(2)
                * (
                    (c2 + 1.0)
                    * (amount * 2.0 - 2.0) + c2
                ) + 2.0
            ) / 2.0
        };

        Self::lerp(current, target, amount)
    }

}
