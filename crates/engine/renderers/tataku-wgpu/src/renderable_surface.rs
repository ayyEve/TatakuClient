use crate::prelude::*;

pub struct RenderableSurface<'a> {
    pub texture: &'a WgpuTextureReference,
    pub size: tataku::Vector2,
    pub clear_color: tataku::Color,
}
impl<'a> RenderableSurface<'a> {
    pub fn new(
        texture: &'a WgpuTextureReference,
        clear_color: tataku::Color,
        size: tataku::Vector2,
    ) -> Self {
        Self {
            texture,
            size,
            clear_color,
        }
    }

    pub fn get_clear_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.clear_color.r.to_f32() as f64,
            g: self.clear_color.g.to_f32() as f64,
            b: self.clear_color.b.to_f32() as f64,
            a: self.clear_color.a.to_f32() as f64
        }
    }
}



pub struct WgpuTextureReference {
    pub view: wgpu::TextureView,
    pub size: wgpu::Extent3d,
}
impl WgpuTextureReference {
    pub fn new(texture: &wgpu::Texture) -> Self {
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            size: texture.size(),
        }
    }
}
