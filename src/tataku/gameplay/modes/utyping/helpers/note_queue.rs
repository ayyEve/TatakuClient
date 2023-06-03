use prelude::modes::utyping::utyping_info::UTypingHitJudgment;

use crate::prelude::*;
use super::super::prelude::*;

#[derive(Default)]
pub struct UTypingNoteQueue {
    pub notes: Vec<UTypingNote>,
    pub index: usize,
}
impl UTypingNoteQueue {
    pub fn new() -> Self { Self::default() }
    pub fn next(&mut self) { self.index += 1; }

    /// this function is a little weird, as it only returns a judgment when the note has been complete, not on the first press
    pub fn check(&mut self, input: char, time: f32, windows: &Vec<(UTypingHitJudgment, Range<f32>)>, manager: &IngameManager) -> Option<UTypingHitJudgment> {
        let Some(current_note) = self.current_note() else { return None };

        let hit_ok = current_note.check_char(&input);
        
        // is this the first hit for this note?
        if current_note.judgment.is_none() {
            let judgment = manager.check_judgment_only(windows, time, current_note.time());

            // this is the first time this note has been hit
            match (hit_ok, judgment) {
                // its not time to hit this note yet
                (_, None) => { return None; }

                // if we missed, 
                (true, Some(&UTypingHitJudgment::Miss)) | (false, _) => {
                    // insta miss
                    current_note.miss(time);

                    self.next();
                    return Some(UTypingHitJudgment::Miss);
                }

                // set the hitjudgment
                (true, Some(j)) => current_note.judgment = Some(*j),
            }
        }
        
        if hit_ok {
            current_note.hit(time, input);
            if current_note.complete() {
                let j = current_note.judgment;
                self.next();
                return j;
            }
        } else {
            // check next note
            if let Some(next_note) = self.next_note() {
                let hit_ok = next_note.check_char(&input);
                let judgment = manager.check_judgment_only(windows, time, next_note.time());

                if hit_ok && (judgment == Some(&UTypingHitJudgment::X100) || judgment == Some(&UTypingHitJudgment::X300)) {
                    // proceed with this note, miss last note.
                    next_note.judgment = judgment.cloned();
                }
            }

            // insta miss
            self.current_note().unwrap().miss(time);

            self.next();
            return Some(UTypingHitJudgment::Miss);
        }

        None
    }
    
    #[inline]
    pub fn current_note(&mut self) -> Option<&mut UTypingNote> {
        self.notes.get_mut(self.index)
    }
    #[inline]
    pub fn next_note(&mut self) -> Option<&mut UTypingNote> {
        self.notes.get_mut(self.index + 1)
    }
}

impl Deref for UTypingNoteQueue {
    type Target = Vec<UTypingNote>;

    fn deref(&self) -> &Self::Target {
        &self.notes
    }
}
impl DerefMut for UTypingNoteQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.notes
    }
}

