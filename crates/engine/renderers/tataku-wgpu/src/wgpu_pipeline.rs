
#[derive(Copy, Clone)]
pub(crate) enum WgpuPipeline<'a> {
    #[cfg(feature="vello")] None,
    Render(&'a wgpu::RenderPipeline),
    Compute(&'a wgpu::ComputePipeline),
}
impl WgpuPipeline<'_> {
    pub fn get_bind_group_layout(&self, index: u32) -> wgpu::BindGroupLayout {
        match self {
            #[cfg(feature="vello")] Self::None => panic!("trying to get bind group for no pipeline!"),
            Self::Render(p) => p.get_bind_group_layout(index),
            Self::Compute(p) => p.get_bind_group_layout(index),
        }
    }
}
impl<'a> From<&'a wgpu::ComputePipeline> for WgpuPipeline<'a> {
    fn from(value: &'a wgpu::ComputePipeline) -> Self {
        Self::Compute(value)
    }
}
impl<'a> From<&'a wgpu::RenderPipeline> for WgpuPipeline<'a> {
    fn from(value: &'a wgpu::RenderPipeline) -> Self {
        Self::Render(value)
    }
}