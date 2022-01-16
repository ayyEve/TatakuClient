use crate::prelude::Vector2;
use std::collections::VecDeque;

// this is essentially osu's math helper

pub const SLIDER_DETAIL_LEVEL:u32 = 50;

pub const PI:f64 = 3.14159274;
pub const TWO_PI:f64 = 6.28318548;

const BEZIER_TOLERANCE:f64 = 0.5;
const BEZIER_TOLERANCE_SQ:f64 = BEZIER_TOLERANCE * BEZIER_TOLERANCE;

struct BezierApproximator {
    count: usize,
    control_points: Vec<Vector2>,
    subdivision_buffer1: Vec<Vector2>,
    subdivision_buffer2: Vec<Vector2>
}
impl BezierApproximator {
    fn new(control_points: Vec<Vector2>) -> Self {
        let count = control_points.len();

        let mut subdivision_buffer1 = Vec::new();
        for _ in 0..count {
            subdivision_buffer1.push(Vector2::zero());
        }

        let mut subdivision_buffer2 = Vec::new();
        for _ in 0..count * 2 - 1 {
            subdivision_buffer2.push(Vector2::zero());
        }

        Self {
            control_points,
            count,

            subdivision_buffer1,
            subdivision_buffer2,
        }
    }

    
    /// Make sure the 2nd order derivative (approximated using finite elements) is within tolerable bounds.
    /// NOTE: The 2nd order derivative of a 2d curve represents its curvature, so intuitively this function
    /// checks (as the name suggests) whether our approximation is _locally_ "flat". More curvy parts
    /// need to have a denser approximation to be more "flat".
    fn is_flat_enough(control_points: &Vec<Vector2>) -> bool {
        for i in 1..control_points.len()-1 {
            if length_squared(control_points[i-1] - control_points[i] * 2.0 + control_points[i+1]) > BEZIER_TOLERANCE_SQ {
                return false;
            }
        }
        true
    }

    
    /// Subdivides n control points representing a bezier curve into 2 sets of n control points, each
    /// describing a bezier curve equivalent to a half of the original curve. Effectively this splits
    /// the original curve into 2 curves which result in the original curve when pieced back together.
    fn subdivide(&mut self, control_points: &Vec<Vector2>, l: &mut Vec<Vector2>, r: &mut Vec<Vector2>) {
        let midpoints = &mut self.subdivision_buffer1;

        for i in 0..self.count {
            midpoints[i] = control_points[i];
        }
        
        for i in 0..self.count {
            l[i] = midpoints[0];
            r[self.count - i - 1] = midpoints[self.count - i - 1];
            
            for j in 0..self.count - i - 1 {
                midpoints[j] = (midpoints[j] + midpoints[j + 1]) / 2.0
            }
        }
    }

    /// This uses <a href="https://en.wikipedia.org/wiki/De_Casteljau%27s_algorithm">De Casteljau's algorithm</a> to obtain
    /// an optimal piecewise-linear approximation of the bezier curve with the same amount of points as there are control points.
    fn approximate(&mut self, control_points: &Vec<Vector2>, output: &mut Vec<Vector2>) {
        let mut l = self.subdivision_buffer2.clone();
        let mut r = self.subdivision_buffer1.clone();

        self.subdivide(&control_points, &mut l, &mut r);

        for i in 0..self.count - 1 {
            l[self.count + i] = r[i + 1];
        }

        output.push(control_points[0]);
        for i in 1..self.count - 1 {
            let index = i * 2;
            let p = (l[index - 1] + l[index] * 2.0 + l[index + 1]) * 0.25;
            output.push(p);
        }

        self.subdivision_buffer2 = l;
        self.subdivision_buffer1 = r;
    }

    
    /// Creates a piecewise-linear approximation of a bezier curve, by adaptively repeatedly subdividing
    /// the control points until their approximation error vanishes below a given threshold.
    fn create_bezier(&mut self) -> Vec<Vector2> {
        let mut output = Vec::new();
        if self.count == 0 {return output}

        // Stack<Vector2[]> toFlatten = new Stack<Vector2[]>();
        // Stack<Vector2[]> freeBuffers = new Stack<Vector2[]>();
        let mut to_flatten = VecDeque::new();
        let mut free_buffers = VecDeque::new();


        // "toFlatten" contains all the curves which are not yet approximated well enough.
        // We use a stack to emulate recursion without the risk of running into a stack overflow.
        // (More specifically, we iteratively and adaptively refine our curve with a 
        // <a href="https://en.wikipedia.org/wiki/Depth-first_search">Depth-first search</a>
        // over the tree resulting from the subdivisions we make.)
        // toFlatten.Push(this.controlPoints.ToArray());
        to_flatten.push_front(self.control_points.clone());

        // Vector2[] leftChild = this.subdivisionBuffer2;
        let mut left_child = self.subdivision_buffer2.clone();
        
        while to_flatten.len() > 0 {
            let mut parent = to_flatten.pop_front().unwrap();

            if BezierApproximator::is_flat_enough(&parent) {
                // If the control points we currently operate on are sufficiently "flat", we use
                // an extension to De Casteljau's algorithm to obtain a piecewise-linear approximation
                // of the bezier curve represented by our control points, consisting of the same amount
                // of points as there are control points.

                // this.Approximate(parent, output);
                self.approximate(&parent, &mut output);
                free_buffers.push_front(parent);
                continue;
            }

            // If we do not yet have a sufficiently "flat" (in other words, detailed) approximation we keep
            // subdividing the curve we are currently operating on.
            let mut right_child = if free_buffers.len() > 0 {
                free_buffers.pop_front().unwrap()
            } else {
                vec![Vector2::zero(); self.count]
            };
            self.subdivide(&parent, &mut left_child, &mut right_child);

            // We re-use the buffer of the parent for one of the children, so that we save one allocation per iteration.
            for i in 0..self.count {
                parent[i] = left_child[i]
            }

            to_flatten.push_front(right_child.clone());
            to_flatten.push_front(parent);
        }

        output.push(self.control_points[self.count - 1]);
        output
    }
}

pub(crate) fn create_bezier(input: Vec<Vector2>) -> Vec<Vector2> {
    let mut b = BezierApproximator::new(input);
    b.create_bezier()
}

pub(crate) fn create_bezier_old(input: Vec<Vector2>) -> Vec<Vector2> {
    let count = input.len();
    let mut working = Vec::new();
    for _ in 0..count {working.push(Vector2::zero())}
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;
    for iteration in 0..points+1 {
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

pub(crate) fn create_bezier_wrong(input: Vec<Vector2>) -> Vec<Vector2> {
    let count = input.len();

    let mut working = Vec::new();
    for _ in 0..count {working.push(Vector2::zero())}
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;

    for iteration in 0..points {
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
        v.y.atan2(v.x)
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
}
impl VectorHelpers for Vector2 {
    fn distance(&self, v2: Vector2) -> f64 {
        distance(*self, v2)
    }
}