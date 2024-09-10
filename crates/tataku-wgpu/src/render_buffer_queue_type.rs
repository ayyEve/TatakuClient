use crate::prelude::*;

pub enum RenderBufferQueueType {
    Vertex(RenderBufferQueue<VertexBuffer>),
    Slider(RenderBufferQueue<SliderRenderBuffer>),
    Flashlight(RenderBufferQueue<FlashlightBuffer>),
}
impl RenderBufferQueueType {
    /// dumps the cached data to the gpu, and returns the buffers which contained that data
    /// also sets up the next recording buffer (creating one if one is not available in the queue)
    pub fn dump_and_next(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) -> Option<RenderBufferType> {
        match self {
            Self::Slider(s) => s.dump_and_next(queue, device).map(|b|RenderBufferType::Slider(b)),
            Self::Vertex(v) => v.dump_and_next(queue, device).map(|b|RenderBufferType::Vertex(b)),
            Self::Flashlight(f) => f.dump_and_next(queue, device).map(|b|RenderBufferType::Flashlight(b)),
        }
    }
    pub fn end(&mut self, queue: &wgpu::Queue) -> Option<RenderBufferType> {
        match self {
            Self::Slider(s) => s.end(queue).map(|b|RenderBufferType::Slider(b)),
            Self::Vertex(v) => v.end(queue).map(|b|RenderBufferType::Vertex(b)),
            Self::Flashlight(f) => f.end(queue).map(|b|RenderBufferType::Flashlight(b)),
        }
    }

    pub fn draw_type(&self) -> LastDrawn {
        match self {
            Self::Slider(_) => LastDrawn::Slider,
            Self::Vertex(_) => LastDrawn::Vertex,
            Self::Flashlight(_) => LastDrawn::Flashlight
        }
    }
}
