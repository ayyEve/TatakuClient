
/// font size helper since f32 isnt hash or eq
pub struct FontSize(f32, u32);
impl FontSize {
    const AMOUNT:f32 = 10.0; // one decimal place
    pub fn new(f: f32) -> Self {
        Self(f, (f * Self::AMOUNT) as u32)
    }
    pub fn u32(&self) -> u32 {
        self.1
    }
    pub fn f32(&self) -> f32 {
        self.0
    }
}
impl std::fmt::Debug for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(f)
    }
}