use crate::prelude::*;
use crate::buffer_queue::RenderBufferQueue;

pub enum RenderBufferQueueType {
    Standard(RenderBufferQueue<shaders::standard::Buffer>),
    Slider(RenderBufferQueue<shaders::slider::Buffer>),
    Flashlight(RenderBufferQueue<shaders::flashlight::Buffer>),
    GaussianBlur(RenderBufferQueue<shaders::gaussian_blur::Buffer>),
    BoxBlur(RenderBufferQueue<shaders::box_blur::Buffer>),
    #[cfg(feature="vello")]
    Vello(RenderBufferQueue<shaders::vello::Buffer>)
}
impl RenderBufferQueueType {
    /// dumps the cached data to the gpu, and returns the buffers which contained that data
    /// also sets up the next recording buffer (creating one if one is not available in the queue)
    pub fn dump_and_next(
        &mut self, 
        queue: &wgpu::Queue, 
        device: &wgpu::Device, 
        pipeline: WgpuPipeline
    ) -> Option<RenderBufferType> {
        match self {
            Self::Standard(v) => v
                .dump_and_next(queue, device, pipeline)
                .map(RenderBufferType::Standard),
            Self::Slider(s) => s
                .dump_and_next(queue, device, pipeline)
                .map(RenderBufferType::Slider),
            Self::Flashlight(f) => f
                .dump_and_next(queue, device, pipeline)
                .map(RenderBufferType::Flashlight),
            Self::GaussianBlur(f) => f
                .dump_and_next(queue, device, pipeline)
                .map(RenderBufferType::GaussianBlur),
            Self::BoxBlur(f) => f
                .dump_and_next(queue, device, pipeline)
                .map(RenderBufferType::BoxBlur),
                
            #[cfg(feature="vello")]
            Self::Vello(f) => f
                .dump_and_next(queue, device, pipeline)
                .map(RenderBufferType::Vello),
        }
    }
    pub fn end(&mut self, queue: &wgpu::Queue) -> Option<RenderBufferType> {
        match self {
            Self::Standard(v) => v
                .end(queue)
                .map(RenderBufferType::Standard),
            Self::Slider(s) => s
                .end(queue)
                .map(RenderBufferType::Slider),
            Self::Flashlight(f) => f
                .end(queue)
                .map(RenderBufferType::Flashlight),
            Self::GaussianBlur(f) => f
                .end(queue)
                .map(RenderBufferType::GaussianBlur),
            Self::BoxBlur(f) => f
                .end(queue)
                .map(RenderBufferType::BoxBlur),

            #[cfg(feature="vello")]
            Self::Vello(f) => f
                .end(queue)
                .map(RenderBufferType::Vello),
        }
    }

    pub fn draw_type(&self) -> PipelineType {
        match self {
            Self::Standard(_) => PipelineType::Standard,
            Self::Slider(_) => PipelineType::Slider,
            Self::Flashlight(_) => PipelineType::Flashlight,
            Self::GaussianBlur(_) => PipelineType::GaussianBlur,
            Self::BoxBlur(_) => PipelineType::BoxBlur,
            
            #[cfg(feature="vello")]
            Self::Vello(_) => PipelineType::Vello,
        }
    }
}
