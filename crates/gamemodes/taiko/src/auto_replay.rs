use crate::prelude::*;

use common::replays::*;
use engine::beatmaps::NoteType;


#[derive(Default)]
pub struct AutoReplay {
    don_presses: u32,
    kat_presses: u32,

    last_hit: f32,
    last_update: f32,
}
impl AutoReplay {
    pub fn update(
        &mut self,
        time: f32,
        queues: &mut [NoteQueue],
        frames: &mut Vec<ReplayAction>
    ) {
        let catching_up = time - self.last_update > 20.0;
        self.last_update = time;

        // if catching_up { trace!("catching up") }

        for queue in queues.iter_mut() {
            let mut queue_index = queue.index;
            let mut note_hit = false;

            for (i, note) in queue.iter_mut().enumerate()
                .skip(queue_index)
                .filter(|(_, note)| time > note.time())
            {
                // note is the note we need to hit

                // if note is a drumroll/spinner, we need to time when to hit it
                // if note is a note, we need to hit it and move on

                // check if we're catching up
                if catching_up {
                    // pretend the note was hit
                    match note {
                        HitObject::Note(note) => note.hit(note.time),
                        HitObject::Drumroll(_) => {},
                        HitObject::Spinner(spinner) => spinner.hit_count = spinner.hits_required,
                    }

                    queue_index = i;
                    continue;
                }

                if note.note_type() != NoteType::Note {
                    let (end_time, hits_to_complete) = match note {
                        HitObject::Note(_) => unreachable!(),
                        HitObject::Drumroll(drumroll) => (
                            drumroll.end_time,
                            50.0,
                        ),
                        HitObject::Spinner(spinner) => (
                            spinner.end_time,
                            spinner.hits_required as f32,
                        ),
                    };

                    // check if time is up
                    if time > end_time {
                        // queue_index = i + 1;
                        continue;
                    }

                    // check if its time to do another hit
                    let duration = end_time - note.time();
                    let time_between_hits = duration / hits_to_complete;

                    // if its not time to do another hit yet
                    if time - self.last_hit < time_between_hits { break }
                }


                // perform the hit
                self.last_hit = time;
                let (is_kat, is_finisher) = match note {
                    HitObject::Note(note) => (
                        matches!(note.hit_type, HitType::Kat),
                        note.base_finisher,
                    ),
                    HitObject::Drumroll(drumroll) => (
                        false,
                        drumroll.base_finisher,
                    ),
                    HitObject::Spinner(spinner) => (
                        matches!(spinner.last_hit.unwrap_or_default(), HitType::Don),
                        false,
                    ),
                };

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
