
pub type Vector3 = cgmath::Vector3<f32>;

#[derive(Copy, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(from = "[f32;2]", into = "[f32;2]")]
pub struct Vector2(cgmath::Vector2<f32>);
impl Vector2 {
    pub const ZERO: Self = Self(cgmath::Vector2::new(0.0, 0.0));
    pub const ONE: Self = Self(cgmath::Vector2::new(1.0, 1.0));

    pub const fn new(x: f32, y: f32) -> Self { Self(cgmath::Vector2::new(x, y)) }

    pub const fn x(&self) -> f32 { self.0.x }
    pub const fn y(&self) -> f32 { self.0.y }
    
    pub const fn with_x(x:f32) -> Self { Self::new(x, 0.0) }
    pub const fn with_y(y:f32) -> Self { Self::new(0.0, y) }

    
    pub fn atan2(self) -> f32 {
        (-self.y).atan2(self.x)
    }
    pub fn atan2_wrong(self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn from_angle(a:f32) -> Self {
        Self::new(a.cos(), a.sin())
    }
    
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(self) -> Self {
        let magnitude = self.length();
        if magnitude == 0.0 { self }
        else { self / magnitude }
    }

    pub fn distance(&self, p2: Self) -> f32 {
        self.distance_squared(p2).sqrt()
    }
    pub fn distance_squared(&self, p2: Self) -> f32 {
        (self.x - p2.x).powi(2) + (self.y - p2.y).powi(2)
    }
    pub fn direction(&self, v2: Self) -> f32 {
        let direction = v2 - *self;
        (direction.x / direction.length()).acos()
    }
    
    // get only this vector's x value
    pub fn x_portion(mut self) -> Self {
        self.y = 0.0;
        self
    }
    // get only this vector's y value
    pub fn y_portion(mut self) -> Self {
        self.x = 0.0;
        self
    }

    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl std::ops::Deref for Vector2 {
    type Target = cgmath::Vector2<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Vector2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[f32;2]> for Vector2 {
    fn from(value: [f32;2]) -> Self {
        Self::new(value[0], value[1])
    }
}
impl Into<[f32;2]> for Vector2 {
    fn into(self) -> [f32;2] {
        [self.x, self.y]
    }
}

#[cfg(feature="graphics")]
impl From<iced::Vector> for Vector2 {
    fn from(value: iced::Vector) -> Self {
        Self::new(value.x, value.y)
    }
}
#[cfg(feature="graphics")]
impl From<iced::Point> for Vector2 {
    fn from(value: iced::Point) -> Self {
        Self::new(value.x, value.y)
    }
}
#[cfg(feature="graphics")]
impl From<iced::Size> for Vector2 {
    fn from(value: iced::Size) -> Self {
        Self::new(value.width, value.height)
    }
}

impl Default for Vector2 {
    fn default() -> Self { Self::new(0.0, 0.0) }
}

impl std::fmt::Display for Vector2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}", self.x, self.y)
    }
}

// negative nancy
impl Neg for Vector2 {
    type Output = Vector2;
    fn neg(self) -> Self::Output {
        Vector2::new(-self.x, -self.y)
    }
}


// fuck you neb, i dont care if this isnt how math works
use std::ops::*;

// add
impl Add<f32> for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x + rhs, self.y + rhs)
    }
}
impl Add<Vector2> for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl AddAssign<f32> for Vector2 {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}
impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        *self = *self + rhs;
    }
}

// sub
impl Sub<f32> for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: f32) -> Self::Output {
        self + -rhs
    }
}
impl Sub<Vector2> for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: Vector2) -> Self::Output {
        self + -rhs
    }
}
impl SubAssign<f32> for Vector2 {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}
impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, rhs: Vector2) {
        *self = *self - rhs;
    }
}

// mul
impl Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x * rhs, self.y * rhs)
    }
}
impl Mul<Vector2> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x * rhs.x, self.y * rhs.y)
    }
}
impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}
impl MulAssign<Vector2> for Vector2 {
    fn mul_assign(&mut self, rhs: Vector2) {
        *self = *self * rhs;
    }
}

// div
impl Div<f32> for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x / rhs, self.y / rhs)
    }
}
impl Div<Vector2> for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x / rhs.x, self.y / rhs.y)
    }
}
impl DivAssign<f32> for Vector2 {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
impl DivAssign<Vector2> for Vector2 {
    fn div_assign(&mut self, rhs: Vector2) {
        *self = *self / rhs;
    }
}

// rem (mod)
impl Rem<f32> for Vector2 {
    type Output = Vector2;
    fn rem(self, rhs: f32) -> Self::Output {
        Vector2::new(self.x % rhs, self.y % rhs)
    }
}
impl Rem<Vector2> for Vector2 {
    type Output = Vector2;
    fn rem(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x % rhs.x, self.y % rhs.y)
    }
}
impl RemAssign<f32> for Vector2 {
    fn rem_assign(&mut self, rhs: f32) {
        *self = *self % rhs;
    }
}
impl RemAssign<Vector2> for Vector2 {
    fn rem_assign(&mut self, rhs: Vector2) {
        *self = *self % rhs;
    }
}
