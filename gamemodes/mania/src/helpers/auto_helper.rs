use crate::prelude::*;

// TODO: document whatever the hell is happening here
pub struct ManiaAutoHelper {
    states: Vec<AutoplayColumnState>,
}
impl ManiaAutoHelper {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
        }
    }

    fn get_keypress(col: usize) -> KeyPress {
        let base_key = KeyPress::Mania1 as u8;
        ((col + base_key as usize) as u8).into()
    }

    pub fn update(&mut self, columns: &[Vec<Box<dyn ManiaHitObject>>], column_indices: &mut [usize], time: f32, list: &mut Vec<ReplayAction>) {
        if self.states.len() != columns.len() {
            let new_len = columns.len();
            self.states.resize(new_len, AutoplayColumnState::default());
            // self.notes_hit.resize(new_len, Vec::new());
        }

        for c in 0..columns.len() {
            let state = &mut self.states[c];
            if state.pressed && time > state.release_time {
                list.push(ReplayAction::Release(Self::get_keypress(c)));
                state.pressed = false;
            }

            if column_indices[c] >= columns[c].len() {continue}

            // catch up??
            for i in column_indices[c]..columns[c].len() {
                let note = &columns[c][i];
                if time > note.end_time(100.0) && !note.was_hit() {
                    column_indices[c] += 1;
                } else {
                    break;
                }
            }

            if column_indices[c] >= columns[c].len() { continue }
            let note = &columns[c][column_indices[c]];
            if time >= note.time() && !note.was_hit() {
                // if the key is already down, dont press it again
                // if timer.0 == note.end_time(15.0) && 
                if state.pressed { continue }

                // press the key, and hold it until the note's end time
                list.push(ReplayAction::Press(Self::get_keypress(c)));
                state.pressed = true;
                if note.note_type() == NoteType::Hold {
                    state.release_time = note.end_time(0.0);
                } else {
                    state.release_time = note.end_time(50.0);
                }
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
struct AutoplayColumnState {
    pressed: bool,
    release_time: f32
}