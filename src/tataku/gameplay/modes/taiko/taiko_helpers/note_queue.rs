use crate::prelude::*;
use super::super::prelude::*;

#[derive(Default)]
pub struct TaikoNoteQueue {
    pub notes: Vec<Box<dyn TaikoHitObject>>,
    pub index: usize,
}
impl TaikoNoteQueue {
    pub fn new() -> Self { Self::default() }
    pub fn done(&self) -> bool { self.index >= self.notes.len() }
    pub fn next(&mut self) { self.index += 1; }

    // some if missed, bool is if miss judgment should be applied
    pub fn check_missed(&mut self, time: f32, miss_window: f32) -> Option<bool> {
        if let Some(note) = self.current_note() {
            if note.end_time(miss_window) < time {
                if note.causes_miss() {
                    note.miss(time);
                    Some(true)
                } else {
                    Some(false)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    #[inline]
    pub fn current_note(&mut self) -> Option<&mut Box<dyn TaikoHitObject>> {
        self.notes.get_mut(self.index)
    }
    #[inline]
    pub fn previous_note(&self) -> Option<&Box<dyn TaikoHitObject>> {
        self.notes.get(self.index - 1)
    }
}

impl Deref for TaikoNoteQueue {
    type Target = Vec<Box<dyn TaikoHitObject>>;

    fn deref(&self) -> &Self::Target {
        &self.notes
    }
}
impl DerefMut for TaikoNoteQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.notes
    }
}

