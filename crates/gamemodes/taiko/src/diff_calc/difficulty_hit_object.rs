use crate::prelude::*;
use engine::beatmaps::NoteType;

#[derive(Clone)]
pub struct DifficultyHitObject {
    pub time: f32,
    pub note_type: NoteType,
    pub is_kat: bool,
    // pub end_time: f32,
    // pub hits_to_complete: u32
}

impl From<&HitObject> for DifficultyHitObject {
    fn from(hit: &HitObject) -> Self {
        let time = hit.time();
        let note_type = hit.note_type();

        let is_kat = match hit {
            HitObject::Note(note) => matches!(note.hit_type, HitType::Kat),
            HitObject::Drumroll(_) => false,
            HitObject::Spinner(spinner) => spinner.last_hit
                .map(|hit_type| matches!(hit_type, HitType::Kat))
                .unwrap_or(false),
        };

        Self {
            time,
            note_type,
            is_kat,
        }
    }
}
