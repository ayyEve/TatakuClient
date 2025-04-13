use crate::prelude::*;

pub enum RenderBufferQueueType {
    Standard(RenderBufferQueue<StandardBuffer>),
    Slider(RenderBufferQueue<SliderRenderBuffer>),
    Flashlight(RenderBufferQueue<FlashlightBuffer>),
    Blur(RenderBufferQueue<BlurBuffer>)
}
impl RenderBufferQueueType {
    /// dumps the cached data to the gpu, and returns the buffers which contained that data
    /// also sets up the next recording buffer (creating one if one is not available in the queue)
    pub fn dump_and_next(&mut self, queue: &Queue, device: &Device, pipeline: WgpuPipeline) -> Option<RenderBufferType> {
        match self {
            Self::Slider(s) => s.dump_and_next(queue, device, pipeline).map(RenderBufferType::Slider),
            Self::Standard(v) => v.dump_and_next(queue, device, pipeline).map(RenderBufferType::Standard),
            Self::Flashlight(f) => f.dump_and_next(queue, device, pipeline).map(RenderBufferType::Flashlight),
            Self::Blur(f) => f.dump_and_next(queue, device, pipeline).map(RenderBufferType::Blur),
        }
    }
    pub fn end(&mut self, queue: &Queue) -> Option<RenderBufferType> {
        match self {
            Self::Slider(s) => s.end(queue).map(RenderBufferType::Slider),
            Self::Standard(v) => v.end(queue).map(RenderBufferType::Standard),
            Self::Flashlight(f) => f.end(queue).map(RenderBufferType::Flashlight),
            Self::Blur(f) => f.end(queue).map(RenderBufferType::Blur),
        }
    }

    pub fn draw_type(&self) -> LastDrawn {
        match self {
            Self::Slider(_) => LastDrawn::Slider,
            Self::Standard(_) => LastDrawn::Standard,
            Self::Flashlight(_) => LastDrawn::Flashlight,
            Self::Blur(_) => LastDrawn::Blur,
        }
    }
}
