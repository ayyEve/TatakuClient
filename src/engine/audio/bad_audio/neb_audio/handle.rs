use std::{sync::atomic::{AtomicBool, Ordering}, time::Instant};

use parking_lot::Mutex;

pub struct AudioHandle {
    pub(super) state_changed: AtomicBool,
    pub(super) state: Mutex<AudioState>,

    pub(super) volume_changed: AtomicBool,
    pub(super) volume: Mutex<f32>,

    pub(super) position_changed: AtomicBool,
    /// New audio position from the start, in milliseconds.
    pub(super) new_position: Mutex<f32>,

    pub(super) playback_speed_changed: AtomicBool,
    /// New audio position from the start, in milliseconds.
    pub(super) playback_speed: Mutex<f64>,

    pub(super) delay: Mutex<f32>,
    pub(super) time: Mutex<f32>,
    pub(super) last_time: Mutex<Instant>,

    pub(super) is_music: AtomicBool,
}

impl AudioHandle {
    pub fn new() -> Self {
        Self {
            state_changed: AtomicBool::new(false),
            state: Mutex::new(AudioState::Paused),

            volume_changed: AtomicBool::new(false),
            volume: Mutex::new(1.0),

            position_changed: AtomicBool::new(false),
            new_position: Mutex::new(0.0),

            playback_speed_changed: AtomicBool::new(false),
            playback_speed: Mutex::new(0.0),

            delay: Mutex::new(0.0),
            time: Mutex::new(0.0),
            last_time: Mutex::new(Instant::now()),

            is_music: AtomicBool::new(false)
        }
    }

    // pub fn state(&self) -> AudioState {
    //     self.state.lock().clone()
    // }

    pub fn set_state(&self, state: AudioState) {
        let mut lock = self.state.lock();

        if *lock == state { return; }

        *lock = state;
        self.state_changed.store(true, Ordering::SeqCst);
    }

    // pub fn volume(&self) -> f32 {
    //     *self.volume.lock()
    // }

    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock() = volume;
        self.volume_changed.store(true, Ordering::SeqCst);
    }

    pub fn current_time(&self) -> f32 {
        *self.time.lock() - *self.delay.lock() + self.last_time.lock().elapsed().as_secs_f32() * 1000.0
    }

    /// Set the current time of the sound, relative to the start of the sound, in milliseconds.
    pub fn set_position(&self, position: f32) {
        *self.new_position.lock() = position;
        self.position_changed.store(true, Ordering::SeqCst);
    }
    /// Set the current time of the sound, relative to the start of the sound, in milliseconds.
    pub fn set_playback_speed(&self, speed: f64) {
        *self.playback_speed.lock() = speed;
        self.playback_speed_changed.store(true, Ordering::SeqCst);
    }

    pub fn play(&self) {
        self.set_state(AudioState::Playing);
    }

    pub fn pause(&self) {
        self.set_state(AudioState::Paused);
    }

    pub fn stop(&self) {
        self.set_state(AudioState::Stopped);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioState {
    Playing,
    Paused,
    Stopped
}
