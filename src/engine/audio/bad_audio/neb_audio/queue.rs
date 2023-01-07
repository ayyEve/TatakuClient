use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;

use super::AudioInstance;

pub struct AudioQueueController {
    has_pending: AtomicBool,
    pending_queue: Mutex<Vec<AudioInstance>>,
}

impl AudioQueueController {
    pub fn add(&self, instance: AudioInstance) {
        self.pending_queue.lock().push(instance);
        self.has_pending.store(true, Ordering::SeqCst);
    }
}

pub struct AudioQueue {
    current_queue: Vec<AudioInstance>,
    controller: Arc<AudioQueueController>,

    pub(super) delay_changed: AtomicBool,
    pub(super) delay: f32,
}
impl AudioQueue {
    pub fn new() -> (Arc<AudioQueueController>, AudioQueue) {
        let controller = Arc::new(AudioQueueController {
            has_pending: AtomicBool::new(false),
            pending_queue: Mutex::new(Vec::new())
        });

        let queue = AudioQueue {
            current_queue: Vec::new(),
            controller: controller.clone(),

            delay_changed: AtomicBool::new(false),
            delay: 0.0,
        };

        (controller, queue)
    }
}

impl AudioQueue {
    pub(super) fn set_delay(&mut self, delay: f32) {
        if self.delay != delay {
            //debug!("Delay changed {}ms -> {}ms", self.delay, delay);

            self.delay = delay;
            self.delay_changed.store(true, Ordering::SeqCst);
        }
    }

    pub(super) fn sync_time(&mut self, now: Instant) {
        for i in self.current_queue.iter_mut() {
            i.sync_time(now);
        }
    }
}

impl Iterator for AudioQueue {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.controller.has_pending.load(Ordering::SeqCst) {
            let mut lock = self.controller.pending_queue.lock();

            self.current_queue.extend(lock.drain(..));
            self.controller.has_pending.store(false, Ordering::SeqCst);
        }

        if self.current_queue.is_empty() {
            return None;
        }

        let mut to_drop = Vec::new();

        let mut sum = 0.0;
        let mut raw_sum = 0.0;
        let delay = if self.delay_changed.load(Ordering::SeqCst) { Some(self.delay) } else { None };

        for (i, sound) in self.current_queue.iter_mut().enumerate() {
            if let Some((raw, val)) = sound.next() {
                sum += val;
                if sound.handle.is_music.load(Ordering::SeqCst) {
                    raw_sum += raw;
                }

                if let Some(delay) = delay {
                    sound.set_delay(delay);
                }
            }
            else {
                // End of stream, close audio instance.
                to_drop.push(i);
            }
        }

        // Remove completed audio instances
        for i in to_drop.into_iter().rev() {
            self.current_queue.remove(i);
        }

        Some((raw_sum, sum))
    }
}
