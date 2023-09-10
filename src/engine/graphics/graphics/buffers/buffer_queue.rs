// use crate::prelude::*;
use wgpu::{ Queue, Device };

pub struct RenderBufferQueue<B:RenderBufferable> {
    pub cpu_cache: B::Cache,
    queued_buffers: Vec<Box<B>>,
    recording_buffer: Option<Box<B>>,
}
impl<B:RenderBufferable> RenderBufferQueue<B> {
    /// create a new buffer queue
    pub fn new() -> Self {
        Self {
            queued_buffers: Vec::with_capacity(3),
            recording_buffer: None,
            cpu_cache: B::Cache::default(),
        }
    }
    
    /// inline helper to create a render buffer on the queue
    pub fn init(mut self, device: &Device) -> Self {
        self.create_render_buffer(device);
        self
    }

    /// set up the buffers to be writable
    pub fn begin(&mut self, mut recorded: Vec<Box<B>>) {
        recorded.iter_mut().for_each(|b|b.reset());
        self.queued_buffers.extend(recorded);

        // the recording buffer can still be some if it was not used in the previous draw call
        if self.recording_buffer.is_none() {
            self.recording_buffer = self.queued_buffers.pop();
        }
    }

    /// finish writing all data to the buffers
    pub fn end(&mut self, queue: &Queue) -> Option<Box<B>> {
        self.dump(queue)
    }

    /// write the data in the cpu cache to the gpu
    pub fn dump(&mut self, queue: &Queue) -> Option<Box<B>> {
        let mut recording_buffer = std::mem::take(&mut self.recording_buffer)?;
        if recording_buffer.should_write() {
            recording_buffer.dump(queue, &self.cpu_cache);
            Some(recording_buffer)
        } else {
            self.queued_buffers.push(recording_buffer);
            None
        }
    }

    /// get the current recording buffer
    pub fn recording_buffer(&mut self) -> Option<&mut Box<B>> { self.recording_buffer.as_mut() }

    /// create a render buffer on the gpu
    pub fn create_render_buffer(&mut self, device: &Device) {
        self.queued_buffers.push(Box::new(B::create_new_buffer(device)));
    }

    /// dump the cached data to the gpu, and set up the next recording buffer, creating a new buffer on the gpu if no existing buffers are available
    pub fn dump_and_next(&mut self, queue: &Queue, device: &Device) -> Option<Box<B>> {
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

    const VTX_PER_BUF: u64;
    const IDX_PER_BUF: u64;

    /// name for this buffer (helpful for debugging)
    fn name() -> &'static str;

    /// reset the render buffer's values to default
    fn reset(&mut self);

    /// dump the cpu cache to the gpu
    fn dump(&mut self, queue: &Queue, cache: &Self::Cache);

    /// whether or not the data should be dumped to the gpu
    fn should_write(&self) -> bool;

    /// create a new buffer on the gpu
    fn create_new_buffer(device: &Device) -> Self;
}
