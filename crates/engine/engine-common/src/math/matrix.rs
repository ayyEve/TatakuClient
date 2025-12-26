use crate::prelude::*;
use cgmath::One;
use cgmath::SquareMatrix;

/// Column Major
pub type Matrix = cgmath::Matrix4<f32>;

pub trait MatrixExt {
    fn identity() -> Self where Self:Sized;
    fn to_raw(&self) -> [[f32; 4]; 4];

    fn inverse(&self) -> Option<Self> where Self: Sized;

    fn from_orient(pos: Vector2) -> Self where Self: Sized;

    fn mul_v2(&self, v: Vector2) -> Vector2;
    fn mul_v3(&self, v: Vector3) -> Vector3;

    // trans!!!!
    fn trans(self, p: Vector2) -> Self;
    fn rot(self, rads: f32) -> Self;
    fn scale(self, s: Vector2) -> Self;
}
impl MatrixExt for Matrix {
    fn identity() -> Self where Self:Sized {
        Matrix::one()
    }
    fn to_raw(&self) -> [[f32; 4]; 4] {
        (*self).into()
    }

    fn inverse(&self) -> Option<Self> {
        self.invert()
    }

    fn from_orient(pos: Vector2) -> Self where Self: Sized {
        let len = pos.x * pos.x + pos.y * pos.y;
        if len == 0.0 { return <Self as MatrixExt>::identity() }

        let len = len.sqrt();
        let c = pos.x / len;
        let s = pos.y / len;
        [
            [c,  -s,   0.0, 0.0],
            [s,   c,   0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ].into()
    }

    fn mul_v3(&self, v: Vector3) -> Vector3 {
        let v = cgmath::Vector4::new(v.x, v.y, v.z, 1.0);
        let v = self * v;
        Vector3::new(v.x, v.y, v.z)
    }
    fn mul_v2(&self, v: Vector2) -> Vector2 {
        let v = cgmath::Vector4::new(v.x, v.y, 0.0, 1.0);
        let v = self * v;
        Vector2::new(v.x, v.y)
    }


    fn trans(self, p: Vector2) -> Self {
        let v3 = Vector3::new(p.x, p.y, 0.0);
        Matrix::from_translation(v3) * self
    }
    fn rot(self, rads: f32) -> Self {
        Matrix::from_angle_z(cgmath::Rad(rads)) * self
    }
    fn scale(self, s: Vector2) -> Self {
        Matrix::from_nonuniform_scale(s.x, s.y, 1.0) * self
    }
}


impl std::ops::Mul<Vector2> for Matrix {
    type Output = Vector2;

    fn mul(self, rhs: Vector2) -> Self::Output {
        self.mul_v2(rhs)
    }
}
