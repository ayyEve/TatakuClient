use std::sync::Arc;

pub struct WgpuTexture {
    pub textures: Arc<Vec<(wgpu::Texture, wgpu::TextureView)>>,
    pub bind_group: wgpu::BindGroup,
}
