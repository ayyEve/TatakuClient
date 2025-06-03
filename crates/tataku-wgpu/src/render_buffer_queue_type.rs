use crate::prelude::*;

pub enum RenderBufferQueueType {
    Standard(RenderBufferQueue<StandardBuffer>),
    Slider(RenderBufferQueue<SliderRenderBuffer>),
    Flashlight(RenderBufferQueue<FlashlightBuffer>),
    GaussianBlur(RenderBufferQueue<GaussianBlurBuffer>),
    BoxBlur(RenderBufferQueue<BoxBlurBuffer>),
}
impl RenderBufferQueueType {
    /// dumps the cached data to the gpu, and returns the buffers which contained that data
    /// also sets up the next recording buffer (creating one if one is not available in the queue)
    pub fn dump_and_next(
        &mut self, 
        queue: &Queue, 
        device: &Device, 
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
        }
    }
    pub fn end(&mut self, queue: &Queue) -> Option<RenderBufferType> {
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
        }
    }

    pub fn draw_type(&self) -> LastPipeline {
        match self {
            Self::Standard(_) => LastPipeline::Standard,
            Self::Slider(_) => LastPipeline::Slider,
            Self::Flashlight(_) => LastPipeline::Flashlight,
            Self::GaussianBlur(_) => LastPipeline::GaussianBlur,
            Self::BoxBlur(_) => LastPipeline::BoxBlur,
        }
    }
}
