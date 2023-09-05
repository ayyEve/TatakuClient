// use crate::prelude::*;
use wgpu::{ Queue, Device };

pub struct RenderBufferQueue<B:RenderBufferable> {
    pub cpu_cache: B::Cache,
    // recorded_buffers: Vec<B>,
    queued_buffers: Vec<B>,
    recording_buffer: Option<B>,
}
impl<B:RenderBufferable> RenderBufferQueue<B> {
    pub fn new() -> Self {
        Self {
            // recorded_buffers: Vec::with_capacity(3),
            queued_buffers: Vec::with_capacity(3),
            recording_buffer: None,
            cpu_cache: B::Cache::default(),
        }
    }

    pub fn begin(&mut self, mut recorded: Vec<B>) {
        // for i in self.recorded_buffers.iter_mut() {
        //     i.reset();
        // }

        recorded.iter_mut().for_each(|b|b.reset());
        self.queued_buffers.extend(recorded);
        self.recording_buffer = self.queued_buffers.pop();
    }

    pub fn end(&mut self, queue: &Queue) -> Option<B> {
        self.dump(queue)
    }

    pub fn dump(&mut self, queue: &Queue) -> Option<B> {
        let mut recording_buffer = std::mem::take(&mut self.recording_buffer)?;
        if recording_buffer.should_write() {
            recording_buffer.dump(queue, &self.cpu_cache);
            // self.recorded_buffers.push(recording_buffer);
            Some(recording_buffer)
        } else {
            self.queued_buffers.push(recording_buffer);
            None
        }
    }

    // pub fn recorded_buffers(&self) -> &Vec<B> { &self.recorded_buffers }
    pub fn recording_buffer(&mut self) -> Option<&mut B> { self.recording_buffer.as_mut() }

    pub fn create_render_buffer(&mut self, device: &Device) {
        self.queued_buffers.push(B::create_new_buffer(device));
    }

    pub fn dump_and_next(&mut self, queue: &Queue, device: &Device) -> Option<B> {
        let dumped = self.dump(queue);
        if self.queued_buffers.is_empty() {
            self.create_render_buffer(device);
        }

        self.recording_buffer = self.queued_buffers.pop();
        dumped
    }
}


pub trait RenderBufferable: Sized {
    type Cache: Default;

    /// reset the render buffer's values to default
    fn reset(&mut self);
    fn dump(&mut self, queue: &Queue, cache: &Self::Cache);

    fn should_write(&self) -> bool;

    fn create_new_buffer(device: &Device) -> Self;
}
