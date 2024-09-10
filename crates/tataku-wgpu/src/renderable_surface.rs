use tataku_client_common::prelude::*;


pub struct RenderableSurface<'a> {
    pub texture: &'a wgpu::TextureView,
    pub size: Vector2,
    pub clear_color: Color,
}
impl<'a> RenderableSurface<'a> {
    pub fn new(texture: &'a wgpu::TextureView, clear_color: Color, size: Vector2) -> Self {
        Self {
            texture,
            size,
            clear_color
        }
    }
    pub fn get_clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.clear_color.r as f64,
            g: self.clear_color.g as f64,
            b: self.clear_color.b as f64,
            a: self.clear_color.a as f64
        }
    }
}