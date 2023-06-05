// use crate::prelude::*;
use cgmath::One;

/// Column Major
pub type Matrix = cgmath::Matrix4<f32>;
pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;

pub trait MatrixHelpers {
    fn identity() -> Self where Self:Sized;
    fn to_raw(&self) -> [[f32; 4]; 4];

    fn from_orient(pos: Vector2) -> Self where Self: Sized;

    fn mul_v3(&self, v: Vector3) -> Vector3;
}
impl MatrixHelpers for Matrix {
    fn identity() -> Self where Self:Sized {
        Matrix::one()
    }
    fn to_raw(&self) -> [[f32; 4]; 4] {
        (*self).into()
    }

    fn from_orient(pos: Vector2) -> Self where Self: Sized {
        let len = pos.x * pos.x + pos.y * pos.y;
        if len == 0.0 { return Self::identity() }

        let len = len.sqrt();
        let c = pos.x / len;
        let s = pos.y / len;
        // [[c, -s, 0.0], [s, c, 0.0]]
        [
            [c as f32, -s as f32, 0.0, 0.0],
            [s as f32,  c as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]
        ].into()
    }

    fn mul_v3(&self, v: Vector3) -> Vector3 {
        let v = cgmath::Vector4::new(v.x, v.y, v.z, 1.0);
        let v = self * v;
        Vector3::new(v.x, v.y, v.z)
    }
}


pub trait Vector2Helpers {
    fn from_other(v: crate::prelude::Vector2) -> Self where Self:Sized;
    fn to_vector3(&self) -> Vector3;
}
impl Vector2Helpers for Vector2 {
    fn from_other(v: crate::prelude::Vector2) -> Self where Self:Sized {
        Self::new(v.x as f32, v.y as f32)
    }
    fn to_vector3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, 0.0)
    }
}
