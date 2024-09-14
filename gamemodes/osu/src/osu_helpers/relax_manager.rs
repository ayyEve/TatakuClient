use crate::prelude::*;

const PRESS_DURATION: f32 = 100.0;
const USABLE_KEYS: &[KeyPress] = &[
    KeyPress::Left,
    KeyPress::Right,
    KeyPress::LeftMouse,
    KeyPress::RightMouse,
];

pub struct RelaxManager {
    key_states: HashMap<KeyPress, KeyState>
}
impl RelaxManager {
    pub fn new() -> Self {
        Self {
            key_states: USABLE_KEYS.iter().map(|k| (*k, KeyState::Unpressed)).collect()
        }
    }

    fn find_free_key(&self) -> Option<KeyPress> {
        for i in USABLE_KEYS {
            if self.key_states.get(i).unwrap().is_free() {
                return Some(*i);
            }
        }

        None
    }

    pub fn check_note(
        &mut self, 
        mouse_pos: Vector2,
        note_end_time: f32, // passing this in because its already calculated
        note: &mut Box<dyn OsuHitObject>,
        note_index: usize,
        state: &mut GameplayStateForUpdate<'_>
    ) {
        // if its time to hit the note, the not hasnt been hit yet, and we're within the note's radius
        if state.time >= note.time() && state.time < note_end_time && !note.was_hit() && note.check_distance(mouse_pos) {
            let Some(key) = self.find_free_key() else { return };

            match note.note_type() {
                NoteType::Note => {
                    *self.key_states.get_mut(&key).unwrap() = KeyState::PressedNote(state.time);
                    state.add_replay_action(ReplayAction::Press(key));
                    // pending_frames.push(ReplayAction::Press(key));
                    // pending_frames.push(ReplayAction::Release(key));
                }
                NoteType::Slider | NoteType::Spinner | NoteType::Hold => {
                    *self.key_states.get_mut(&key).unwrap() = KeyState::PressedSlider(note_index);
                    state.add_replay_action(ReplayAction::Press(key));

                    // // make sure we're not already holding
                    // if let Some(false) = state.key_counter.keys.get(&key).map(|a| a.held) {
                    //     state.add_replay_action(ReplayAction::Press(key));
                    //     // pending_frames.push(ReplayAction::Press(key));
                    // }
                }
            }
        }

        if state.time >= note_end_time && !note.was_hit() {
            // let key = KeyPress::LeftMouse;

            match note.note_type() {
                NoteType::Note => {}
                NoteType::Slider | NoteType::Spinner | NoteType::Hold => {

                    for (key, key_state) in self.key_states.iter_mut() {
                        let KeyState::PressedSlider(index) = key_state else { continue };
                        if *index != note_index { continue }
                        *key_state = KeyState::Unpressed;
                        state.add_replay_action(ReplayAction::Release(*key));
                        break;
                    }

                    // assume we're holding i guess?
                    // state.add_replay_action(ReplayAction::Release(key));
                    // pending_frames.push(ReplayAction::Release(key));
                }
            }
        }
    }

    pub fn update(&mut self, time: f32) {
        for state in self.key_states.values_mut() {
            let KeyState::PressedNote(pressed_time) = state else { continue };

            if time > *pressed_time + PRESS_DURATION {
                *state = KeyState::Unpressed;
            }
        }
    }

    pub fn key_pressed(&mut self, key: KeyPress) {
        let Some(state) = self.key_states.get_mut(&key) else { return };
        *state = KeyState::PressedManual;
    }
    pub fn key_released(&mut self, key: KeyPress) {
        let Some(state) = self.key_states.get_mut(&key) else { return };
        if *state == KeyState::PressedManual {
            *state = KeyState::Unpressed;
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum KeyState {
    /// key is not pressed
    #[default]
    Unpressed,
    
    /// key is pressed by us (RelaxManager) for a note
    PressedNote(f32),

    /// key is pressed by us (RelaxManager) for a slider
    PressedSlider(usize),

    /// key is pressed by user
    PressedManual,
}
impl KeyState {
    fn is_free(&self) -> bool {
        let &Self::Unpressed = self else { return false };
        true
    }
}

