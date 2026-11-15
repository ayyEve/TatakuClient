use crate::WgpuPipeline;

pub(crate) trait RenderBufferable: Sized {
    type Cache: Default;

    const VTX_PER_BUF: u64;
    const IDX_PER_BUF: u64;

    /// reset the render buffer's values to default
    fn reset(&mut self);

    /// dump the cpu cache to the gpu
    fn dump(&mut self, queue: &wgpu::Queue, cache: &mut Self::Cache);

    /// whether or not the data should be dumped to the gpu
    fn should_write(&self) -> bool;

    /// create a new buffer on the gpu
    fn create_new_buffer(device: &wgpu::Device, pipeline: WgpuPipeline) -> Self;
}
