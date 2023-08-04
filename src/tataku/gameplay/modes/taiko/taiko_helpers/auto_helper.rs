use crate::prelude::*;
use super::super::prelude::*;

pub struct TaikoAutoHelper {
    don_presses: u32,
    kat_presses: u32,

    last_hit: f32,
    last_update: f32,
}
impl TaikoAutoHelper {
    pub fn new() -> Self {
        Self {
            don_presses: 0, 
            kat_presses: 0, 
            last_hit: 0.0, 
            last_update: 0.0
        }
    }

    pub fn update(&mut self, time: f32, queues: &mut Vec<TaikoNoteQueue>, frames: &mut Vec<ReplayAction>) {
        let catching_up = time - self.last_update > 20.0;
        self.last_update = time;

        // if catching_up { trace!("catching up") }

        for queue in queues.iter_mut() {
            let mut queue_index = queue.index;
            let mut note_hit = false;

            for (i, note) in queue.iter_mut().enumerate().skip(queue_index).filter(|(_, note)|time > note.time() && !note.was_hit()) {
                // note is the note we need to hit

                // if note is a drumroll/spinner, we need to time when to hit it
                // if note is a note, we need to hit it and move on
                
                // check if we're catching up
                if catching_up {
                    // pretend the note was hit
                    note.force_hit();
                    queue_index = i;
                    continue;
                }

                // // otherwise it spams sliders even after it has finished
                // if let NoteType::Slider = note.note_type() {
                //     if time > note.end_time(0.0) {
                //         if i == queue_index {
                //             queue_index += 1;
                //         }
                //         continue 'notes;
                //     }
                // }

                if note.note_type() != NoteType::Note {
                    // this is a drumroll or a spinner
                    let end_time = note.end_time(0.0);

                    // check if time is up
                    if time > end_time { 
                        // queue_index = i + 1;
                        continue;
                    }

                    // check if its time to do another hit
                    let duration = end_time - note.time();
                    let time_between_hits = duration / (note.hits_to_complete() as f32);
                    
                    // if its not time to do another hit yet
                    if time - self.last_hit < time_between_hits { break }
                }


                // perform the hit
                self.last_hit = time;
                let is_kat = note.is_kat();
                let is_finisher = note.is_finisher();

                if is_finisher {
                    if is_kat {
                        frames.push(ReplayAction::Press(KeyPress::LeftKat));
                        frames.push(ReplayAction::Press(KeyPress::RightKat));
                    } else {
                        frames.push(ReplayAction::Press(KeyPress::LeftDon));
                        frames.push(ReplayAction::Press(KeyPress::RightDon));
                    }
                } else {
                    let side = (self.don_presses + self.kat_presses) % 2;
                    match (is_kat, side) {
                        // kat, left side
                        (true, 0) => frames.push(ReplayAction::Press(KeyPress::LeftKat)),

                        // kat, right side
                        (true, 1) => frames.push(ReplayAction::Press(KeyPress::RightKat)),

                        // don, left side
                        (false, 0) => frames.push(ReplayAction::Press(KeyPress::LeftDon)),
                        
                        // don, right side
                        (false, 1) => frames.push(ReplayAction::Press(KeyPress::RightDon)),

                        // shouldnt happen
                        _ => {}
                    }
                }

                if is_kat {
                    self.kat_presses += 1;
                } else {
                    self.don_presses += 1;
                }

                note_hit = true;
                break;
            }

            queue.index = queue_index;
            if note_hit { return }
        }

    }
}
