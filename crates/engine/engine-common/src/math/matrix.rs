use crate::prelude::*;

type Vector4 = glam::f32::Vec4;
pub type Matrix = glam::f32::Mat4;

pub trait MatrixExt {
    fn mul_v2(&self, v: Vector2) -> Vector2;

    // trans!!!!
    fn trans(self, p: Vector2) -> Self;
    fn rot(self, rads: f32) -> Self;
    fn scale(self, s: Vector2) -> Self;
}
impl MatrixExt for Matrix {
    fn mul_v2(&self, v: Vector2) -> Vector2 {
        let v = Vector4::new(v.x, v.y, 0.0, 1.0);
        let v = self * v;
        Vector2::new(v.x, v.y)
    }

    fn trans(self, p: Vector2) -> Self {
        let v3 = Vector3::new(p.x, p.y, 0.0);
        Matrix::from_translation(v3) * self
    }
    fn rot(self, rads: f32) -> Self {
        Matrix::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), rads) * self
    }
    fn scale(self, s: Vector2) -> Self {
        Matrix::from_scale(Vector3::new(s.x, s.y, 1.0)) * self
    }
}

impl std::ops::Mul<Vector2> for Matrix {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Self::Output {
        self.mul_v2(rhs)
    }
}
