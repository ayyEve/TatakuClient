use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use super::{ Sound, AudioHandle, AudioState, utils::interleave };

pub struct AudioInstance {
    sound: Sound,
    stream_sample_rate: u32,

    interpolation_value: f64,
    current_frame: Vec<f32>,
    next_frame: Vec<f32>,
    output_buffer: Vec<f32>,

    interleaved_samples: Vec<f32>,

    current_index: usize,
    playback_speed: f64,
    state: AudioState,
    volume: f32,

    pub(super) handle: Arc<AudioHandle>,
}

impl AudioInstance {
    pub fn new(sound: Sound, stream_sample_rate: u32, playback_speed: f64) -> Self {
        // let now = Instant::now();

        let interleaved = if sound.channels == 1 {
            let mut vec = sound.samples.as_ref().clone();

            vec.push(vec[0].clone());

            interleave(&vec)

        } else {
            interleave(&sound.samples)
        };

        let mut temp_iter = interleaved.iter();

        let current_frame = temp_iter.by_ref().take(sound.channels).cloned().collect();
        let next_frame = temp_iter.take(sound.channels).cloned().collect();
        let handle = Arc::new(AudioHandle::new());

        // debug!("took {:2}ms to interleave", now.elapsed().as_secs_f32() * 1000.0);

        Self {
            sound,
            stream_sample_rate,

            interpolation_value: 0.0,
            current_frame,
            next_frame,
            output_buffer: Vec::new(),

            interleaved_samples: interleaved,

            current_index: 0,
            playback_speed,
            state: AudioState::Paused,
            volume: 1.0,

            handle
        }
    }

    pub fn set_delay(&mut self, delay: f32) {
        *self.handle.delay.lock() = delay;
    }

    /// Returns the current time in milliseconds from the start of the audio instance
    pub fn current_time(&self) -> f32 {
        self.current_index as f32 / (self.sound.sample_rate as f32 * self.sound.channels as f32) * 1000.0
    }

    fn update(&mut self) {
        if self.handle.state_changed.load(Ordering::SeqCst) {
            self.state = self.handle.state.lock().clone();

            self.handle.state_changed.store(false, Ordering::SeqCst);
        }

        if self.handle.volume_changed.load(Ordering::SeqCst) {
            self.volume = self.handle.volume.lock().clone();

            self.handle.volume_changed.store(false, Ordering::SeqCst);
        }

        if self.handle.playback_speed_changed.load(Ordering::SeqCst) {
            self.playback_speed = self.handle.playback_speed.lock().clone();

            self.handle.playback_speed_changed.store(false, Ordering::SeqCst);
        }

        if self.handle.position_changed.load(Ordering::SeqCst) {
            let new_position = {
                let mut lock = self.handle.new_position.lock();
                let mut new_position = 0.0;
                std::mem::swap(&mut *lock, &mut new_position);
                new_position
            };

            let seconds = new_position / 1000.0;
            let index = seconds * self.sound.sample_rate as f32 * self.sound.channels as f32;

            self.current_index = index.floor() as usize;
            self.next_frame();
            self.current_index -= 1;
            self.next_frame();

            self.handle.position_changed.store(false, Ordering::SeqCst);
        }
    }

    fn next_frame(&mut self) {
        self.current_index += self.sound.channels;

        std::mem::swap(&mut self.current_frame, &mut self.next_frame);
        self.next_frame.clear();

        for i in 0..self.sound.channels {
            if let Some(&s) = self.interleaved_samples.get(self.current_index + i) {
                self.next_frame.push(s);
            }
            else {
                break;
            }
        }
    }

    pub fn sync_time(&mut self, instant:Instant) {
        // *self.handle.last_time.lock() = Instant::now();
        *self.handle.time.lock() = self.current_time();
        *self.handle.last_time.lock() = instant;
    }
}

impl Iterator for AudioInstance {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        self.update();

        match self.state {
            AudioState::Paused => return Some((0.0, 0.0)),
            AudioState::Stopped => return None,
            AudioState::Playing => {},
        }

        if self.sound.sample_rate == self.stream_sample_rate {
            // No conversion and interpolation necessary
            let sample = *self.interleaved_samples.get(self.current_index)?;
            self.current_index += 1;
            return Some((sample, sample * self.volume));
        }

        if !self.output_buffer.is_empty() {
            // No need to recalculate yet, just use what we already have
            let sample = self.output_buffer.remove(0);
            return Some((sample, sample * self.volume));
        }

        let effective_stream_rate = self.stream_sample_rate as f64 / self.playback_speed;
        let ratio = self.sound.sample_rate as f64 / effective_stream_rate;
        while self.interpolation_value >= 1.0 {
            self.next_frame();
            self.interpolation_value -= 1.0;
        }

        let temp_value = self.interpolation_value;

        self.output_buffer.extend(self.current_frame.iter().zip(self.next_frame.iter())
            .map(|(&current, &next)| (next - current) * temp_value as f32 + current));

        self.interpolation_value += ratio;

        if self.output_buffer.is_empty() {
            None
        }
        else {
            let sample = self.output_buffer.remove(0);
            Some((sample, sample * self.volume))
        }
    }
}
