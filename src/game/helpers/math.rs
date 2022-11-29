use std::ops::Add;

use crate::prelude::Vector2;

// this is essentially osu's math helper

pub const SLIDER_DETAIL_LEVEL:u32 = 50;

pub const PI:f64 = 3.14159274;
pub const TWO_PI:f64 = 6.28318548;

pub(crate) fn create_bezier(input: Vec<Vector2>, wrong: bool) -> Vec<Vector2> {
    let count = input.len();
    let mut working = vec![Vector2::zero(); count];
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;
    for iteration in 0..(if wrong {points} else {points+1}) {
        for i in 0..count {working[i] = input[i]}
        for level in 0..count {
            for i in 0..count - level - 1 {
                working[i] = Vector2::lerp(working[i], working[i+1], iteration as f64 / points as f64);
            }
        }
        output.push(working[0]);
    }
    output
}

fn length_squared(p:Vector2) -> f64 {
    p.x * p.x + p.y * p.y
}

fn distance(p1:Vector2, p2:Vector2) -> f64 {
    let num = p1.x - p2.x;
    let num2 = p1.y - p2.y;
    let num3 = num * num + num2 * num2;
    num3.sqrt()
}

pub fn is_straight_line(a:Vector2, b:Vector2, c:Vector2) -> bool {
    (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y) == 0.0
}

pub fn circle_t_at(p:Vector2, c:Vector2) -> f64 {
    (p.y - c.y).atan2(p.x - c.x)
}

/// Circle through 3 points
/// http://en.wikipedia.org/wiki/Circumscribed_circle#Cartesian_coordinates
pub fn circle_through_points(a:Vector2, b:Vector2, c:Vector2) -> (Vector2, f64, f64, f64) {
    let d = (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) * 2.0;
    let a_mag_sq = length_squared(a);
    let b_mag_sq = length_squared(b);
    let c_mag_sq = length_squared(c);

    let center = Vector2::new(
        (a_mag_sq * (b.y - c.y) + b_mag_sq * (c.y - a.y) + c_mag_sq * (a.y - b.y)) / d,
        (a_mag_sq * (c.x - b.x) + b_mag_sq * (a.x - c.x) + c_mag_sq * (b.x - a.x)) / d
    );
    let radius = distance(center, a);

    let t_initial = circle_t_at(a, center);
    let mut t_mid = circle_t_at(b, center);
    let mut t_final = circle_t_at(c, center);

    while t_mid < t_initial {t_mid += TWO_PI}
    while t_final < t_initial {t_final += TWO_PI}
    if t_mid > t_final {t_final -= TWO_PI}

    (center, radius, t_initial, t_final)
}


pub(crate) fn circle_point(center:Vector2, radius:f64, a:f64) -> Vector2 {
    Vector2::new(
        a.cos() * radius,
        a.sin() * radius
    ) + center
}


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


// help
pub trait Interpolation {
    fn lerp(current: Self, target: Self, amount: f64) -> Self;

    // helpers since many of the easing fns are just different powers
    fn ease_in_exp(current: Self, target: Self, amount: f64, pow: i32) -> Self;
    fn ease_out_exp(current: Self, target: Self, amount: f64, pow: i32) -> Self;
    fn ease_inout_exp(current: Self, target: Self, amount: f64, pow: i32) -> Self;

    // sine
    fn easein_sine(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_sine(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_sine(current:Self, target: Self, amount: f64) -> Self;

    // quadratic
    fn easein_quadratic(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_quadratic(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_quadratic(current:Self, target: Self, amount: f64) -> Self;

    // cubic
    fn easein_cubic(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_cubic(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_cubic(current:Self, target: Self, amount: f64) -> Self;

    // quartic
    fn easein_quartic(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_quartic(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_quartic(current:Self, target: Self, amount: f64) -> Self;

    // quintic
    fn easein_quintic(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_quintic(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_quintic(current:Self, target: Self, amount: f64) -> Self;

    // exponential
    fn easein_exponential(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_exponential(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_exponential(current:Self, target: Self, amount: f64) -> Self;

    // circular
    fn easein_circular(current:Self, target: Self, amount: f64) -> Self;
    fn easeout_circular(current:Self, target: Self, amount: f64) -> Self;
    fn easeinout_circular(current:Self, target: Self, amount: f64) -> Self;

    // back
    // todo! come up with better names than c1 and c3
    fn easein_back(current:Self, target: Self, amount: f64, c1:f64, c3: f64) -> Self;
    fn easeout_back(current:Self, target: Self, amount: f64, c1:f64, c3: f64) -> Self;
    fn easeinout_back(current:Self, target: Self, amount: f64, c1:f64, c3: f64) -> Self;

    // skipping elastic and bounce bc they kinda suck
}
impl<T> Interpolation for T where T: Copy + std::ops::Add<Output=T> + std::ops::Sub<Output=T> + std::ops::Mul<f64, Output=T> {
    fn lerp(current:T, target:T,  amount:f64) -> T {
        current + (target - current) * amount
    }

    // helpers
    fn ease_in_exp(current:T, target:T, amount:f64, pow:i32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, amount.powi(pow))
    }
    fn ease_out_exp(current:T, target:T, amount:f64, pow:i32) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - (1.0 - amount).powi(pow))
    }
    fn ease_inout_exp(current:T, target:T, amount:f64, pow:i32) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount < 0.5 {
            2.0f64.powi(pow - 1) * amount.powi(pow)
        } else {
            1.0 - (-2.0 * amount + 2.0).powi(pow) / 2.0
        };
        Self::lerp(current, target, amount)
    }

    // sine
    fn easein_sine(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - ((amount * PI) / 2.0).cos())
    }
    fn easeout_sine(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, ((amount * PI) / 2.0).sin())
    }
    fn easeinout_sine(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, -((amount * PI).cos() - 1.0) / 2.0)
    }

    // quad
    fn easein_quadratic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 2)
    }
    fn easeout_quadratic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 2)
    }
    fn easeinout_quadratic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 2)
    }

    // cubic
    fn easein_cubic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 3)
    }
    fn easeout_cubic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 3)
    }
    fn easeinout_cubic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 3)
    }

    // quart
    fn easein_quartic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 4)
    }
    fn easeout_quartic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 4)
    }
    fn easeinout_quartic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 4)
    }

    // quint
    fn easein_quintic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_in_exp(current, target, amount, 5)
    }
    fn easeout_quintic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_out_exp(current, target, amount, 5)
    }
    fn easeinout_quintic(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        Self::ease_inout_exp(current, target, amount, 5)
    }

    // expo
    fn easein_exponential(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount == 0.0 {0.0} else {
            2f64.powf(amount * 10.0  - 10.0)
        };
        Self::lerp(current, target, amount)
    }
    fn easeout_exponential(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount == 1.0 {1.0} else {
            1.0 - 2f64.powf(amount * -10.0)
        };
        Self::lerp(current, target, amount)
    }
    fn easeinout_exponential(current:T, target:T, amount:f64) -> T {
        check_bounds!(current, target, amount);
        let amount =
            if amount == 0.0 {0.0}
            else if amount == 1.0 {1.0}
            else if amount < 0.5 {
                2f64.powf(20.0 * amount - 10.0) / 2.0
            } else {
                (2.0 - 2f64.powf(-20.0 * amount + 10.0)) / 2.0
            };
        Self::lerp(current, target, amount)
    }

    // circular
    fn easein_circular(current:T, target: T, amount: f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - (1.0 - amount.powi(2)).sqrt())
    }
    fn easeout_circular(current:T, target: T, amount: f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 - ((amount - 1.0).powi(2)).sqrt())
    }
    fn easeinout_circular(current:T, target: T, amount: f64) -> T {
        check_bounds!(current, target, amount);
        let amount = if amount < 0.5 {
            (1.0 - (1.0 - (2.0 * amount).powi(2)).sqrt()) / 2.0
        } else {
            ((1.0 - (-2.0 * amount + 2.0).powi(2)).sqrt() + 1.0) / 2.0
        };
        Self::lerp(current, target, amount)
    }

    // back
    fn easein_back(current:T, target: T, amount: f64, c1:f64, c3:f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, c3 * amount.powi(3) - c1 * amount.powi(2))
    }
    fn easeout_back(current:T, target: T, amount: f64, c1:f64, c3: f64) -> T {
        check_bounds!(current, target, amount);
        Self::lerp(current, target, 1.0 + c3 * (amount - 1.0).powi(3) - c1 * (amount - 1.0).powi(2))
    }
    fn easeinout_back(current:T, target: T, amount: f64, _c1:f64, c2: f64) -> T {
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


pub trait VectorHelpers {
    fn atan2(v:Vector2) -> f64 {
        (-v.y).atan2(v.x)
    }

    fn from_angle(a:f64) -> Vector2 {
        Vector2::new(a.cos(), a.sin())
    }

    fn magnitude(v: Vector2) -> f64 {
        (v.x * v.x + v.y * v.y).sqrt()
    }

    fn normalize(v: Vector2) -> Vector2 {
        let magnitude = Vector2::magnitude(v);
        if magnitude == 0.0 { v }
        else { v / magnitude }
    }

    fn distance(&self, v2: Vector2) -> f64;
    fn direction(&self, v2: Vector2) -> f64;

    fn x(self) -> Vector2;
    fn y(self) -> Vector2;

    fn cross(self, other: Vector2) -> f64;
    fn dot(self, other: Vector2) -> f64;
}
impl VectorHelpers for Vector2 {
    fn distance(&self, v2: Vector2) -> f64 {
        distance(*self, v2)
    }
    fn direction(&self, v2: Vector2) -> f64 {
        let direction = v2 - *self;
        (direction.x / Self::magnitude(direction)).acos()
    }

    // get only this vector's x value
    fn x(mut self) -> Vector2 {
        self.y = 0.0;
        self
    }
    // get only this vector's y value
    fn y(mut self) -> Vector2 {
        self.x = 0.0;
        self
    }

    fn cross(self, other: Vector2) -> f64 {
        self.x * other.y - self.y * other.x
    }
    fn dot(self, other: Vector2) -> f64 {
        self.x * other.x + self.y * other.y
    }
}


pub trait Sum<T> {
    fn sum(&mut self) -> T;
}
impl<T:Add<Output = T> + Default> Sum<T> for dyn Iterator<Item = T> {
    fn sum(&mut self) -> T {
        self.fold(T::default(), |sum, next| sum + next)
    }
}
impl<T:Add<Output = T> + Default + Copy> Sum<T> for Vec<T> {
    fn sum(&mut self) -> T {
        self.iter().fold(T::default(), |sum, next| sum + *next)
    }
}
impl<T:Add<Output = T> + Default + Copy> Sum<T> for [T] {
    fn sum(&mut self) -> T {
        self.iter().fold(T::default(), |sum, next| sum + *next)
    }
}
