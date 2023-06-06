use crate::prelude::*;

// this is essentially osu's math helper
pub const SLIDER_DETAIL_LEVEL:u32 = 50;
pub const TWO_PI:f32 = PI * 2.0;

pub(crate) fn create_bezier(input: Vec<Vector2>, wrong: bool) -> Vec<Vector2> {
    let count = input.len();
    let mut working = vec![Vector2::ZERO; count];
    let mut output = Vec::new();

    let points = SLIDER_DETAIL_LEVEL * count as u32;
    for iteration in 0..(if wrong {points} else {points+1}) {
        for i in 0..count {working[i] = input[i]}
        for level in 0..count {
            for i in 0..count - level - 1 {
                working[i] = Vector2::lerp(working[i], working[i+1], iteration as f32 / points as f32);
            }
        }
        output.push(working[0]);
    }
    output
}

pub fn is_straight_line(a:Vector2, b:Vector2, c:Vector2) -> bool {
    (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y) == 0.0
}

pub fn circle_t_at(p:Vector2, c:Vector2) -> f32 {
    (p.y - c.y).atan2(p.x - c.x)
}

/// Circle through 3 points
/// http://en.wikipedia.org/wiki/Circumscribed_circle#Cartesian_coordinates
pub fn circle_through_points(a:Vector2, b:Vector2, c:Vector2) -> (Vector2, f32, f32, f32) {
    let d = (a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) * 2.0;
    let a_mag_sq = a.length_squared();
    let b_mag_sq = b.length_squared();
    let c_mag_sq = c.length_squared();

    let center = Vector2::new(
        (a_mag_sq * (b.y - c.y) + b_mag_sq * (c.y - a.y) + c_mag_sq * (a.y - b.y)) / d,
        (a_mag_sq * (c.x - b.x) + b_mag_sq * (a.x - c.x) + c_mag_sq * (b.x - a.x)) / d
    );
    let radius = center. distance(a);

    let t_initial = circle_t_at(a, center);
    let mut t_mid = circle_t_at(b, center);
    let mut t_final = circle_t_at(c, center);

    while t_mid < t_initial {t_mid += TWO_PI}
    while t_final < t_initial {t_final += TWO_PI}
    if t_mid > t_final {t_final -= TWO_PI}

    (center, radius, t_initial, t_final)
}


pub(crate) fn circle_point(center:Vector2, radius:f32, a:f32) -> Vector2 {
    Vector2::new(
        a.cos() * radius,
        a.sin() * radius
    ) + center
}