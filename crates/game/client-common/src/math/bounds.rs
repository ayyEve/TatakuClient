use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Bounds {
    pub pos: Vector2,
    pub size: Vector2,
}
impl Bounds {
    pub const fn new(pos: Vector2, size: Vector2) -> Self {
        Self { 
            pos, 
            size, 
        }
    }
    /// check if these bounds contain a point
    pub fn contains(&self, p: Vector2) -> bool {
        p.x > self.pos.x && p.x < self.pos.x + self.size.x 
        && p.y > self.pos.y && p.y < self.pos.y + self.size.y
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let max_pos = Vector2::new(
            self.pos.x.max(other.pos.x),
            self.pos.y.max(other.pos.y)
        );

        let p2 = self.pos + self.size;
        let other_2 = other.pos + other.size;

        let min_pos = Vector2::new(
            p2.x.min(other_2.x),
            p2.y.min(other_2.y)
        );

        (max_pos.x < min_pos.x && max_pos.y < min_pos.y).then(|| Self::new(
            max_pos,
            min_pos - max_pos,
        ))
    }

    pub fn into_quad(&self) -> [Vector2; 4] {
        let tl = self.pos;
        let tr = self.pos + self.size.x_portion();
        let bl = self.pos + self.size.y_portion();
        let br = self.pos + self.size;
        [tl, tr, bl, br]
    }

    pub fn into_scissor(&self) -> [f32; 4] {
        [
            self.pos.x, self.pos.y,
            self.size.x, self.size.y
        ]
    }

    pub fn area(&self) -> f32 {
        self.size.x * self.size.y
    }
    pub fn has_area(&self) -> bool {
        self.size.x > 0.0 && self.size.y > 0.0
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bounds (pos: {}, size: {})", self.pos, self.size )
    }
}


#[cfg(feature="ui")]
impl From<taffy::geometry::Rect<f32>> for Bounds {
    fn from(value: taffy::geometry::Rect<f32>) -> Self {
        Self::new(
            Vector2::new(value.left, value.top),
            Vector2::new(value.right - value.left, value.bottom - value.top)
        )
    }
}

impl From<[f32;4]> for Bounds {
    fn from([x, y, w, h]: [f32;4]) -> Self {
        Self::new(
            Vector2::new(x, y),
            Vector2::new(w, h)
        )
    }
}

impl std::ops::Mul<Bounds> for Matrix {
    type Output = Bounds;

    fn mul(self, rhs: Bounds) -> Self::Output {
        let br = rhs.pos + rhs.size;
        let pos = self * rhs.pos;

        Bounds {
            pos,
            size: self * br - pos,
        }
    }
}
