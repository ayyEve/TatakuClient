use crate::prelude::*;

pub enum RenderBufferQueueType {
    Standard(RenderBufferQueue<StandardBuffer>),
    Slider(RenderBufferQueue<SliderRenderBuffer>),
    Flashlight(RenderBufferQueue<FlashlightBuffer>),
}
impl RenderBufferQueueType {
    /// dumps the cached data to the gpu, and returns the buffers which contained that data
    /// also sets up the next recording buffer (creating one if one is not available in the queue)
    pub fn dump_and_next(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) -> Option<RenderBufferType> {
        match self {
            Self::Slider(s) => s.dump_and_next(queue, device).map(RenderBufferType::Slider),
            Self::Standard(v) => v.dump_and_next(queue, device).map(RenderBufferType::Standard),
            Self::Flashlight(f) => f.dump_and_next(queue, device).map(RenderBufferType::Flashlight),
        }
    }
    pub fn end(&mut self, queue: &wgpu::Queue) -> Option<RenderBufferType> {
        match self {
            Self::Slider(s) => s.end(queue).map(RenderBufferType::Slider),
            Self::Standard(v) => v.end(queue).map(RenderBufferType::Standard),
            Self::Flashlight(f) => f.end(queue).map(RenderBufferType::Flashlight),
        }
    }

    pub fn draw_type(&self) -> LastDrawn {
        match self {
            Self::Slider(_) => LastDrawn::Slider,
            Self::Standard(_) => LastDrawn::Standard,
            Self::Flashlight(_) => LastDrawn::Flashlight
        }
    }
}
