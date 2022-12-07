use crate::prelude::*;
use super::super::prelude::*;

#[derive(Clone)]
pub struct DifficultyHitObject {
    pub time: f32,
    pub note_type: NoteType,
    pub is_kat: bool,
    pub end_time: f32,
    pub hits_to_complete: u32
}

impl DifficultyHitObject {
    pub fn new(base:&Box<dyn TaikoHitObject>) -> Self {
        let time = base.time();
        let end_time = base.end_time(0.0);
        let hits_to_complete = base.hits_to_complete();

        Self {
            time,
            end_time,
            hits_to_complete,
            note_type: base.note_type(),
            is_kat: base.is_kat(),
        }
    }
}