use crate::prelude::*;

use tataku::Vector2;
use common::replays::*;
use engine::gameplay::gameplay_manager::GameplayUpdateShell;

const PRESS_DURATION: f32 = 100.0;
const USABLE_KEYS: &[KeyPress] = &[
    KeyPress::Left,
    KeyPress::Right,
    KeyPress::LeftMouse,
    KeyPress::RightMouse,
];

pub struct RelaxPlay {
    key_states: HashMap<KeyPress, KeyState>
}

impl Default for RelaxPlay {
    fn default() -> Self {
        let key_states = USABLE_KEYS.iter()
            .map(|&k| (k, KeyState::Unpressed))
            .collect();

        Self {
            key_states,
        }
    }
}

impl RelaxPlay {
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
        note: &mut HitObject,
        note_index: usize,
        coords: &OsuCoords,
        state: &mut GameplayUpdateShell<'_>
    ) {
        let cs = coords.cs / coords.scale;

        // if its time to hit the note, the not hasnt been hit yet, and we're within the note's radius
        if state.time >= note.time() && state.time < note_end_time {
            let hit = match note {
                HitObject::Circle(note) => {
                    let within_hit_range = note.pos.distance_squared(mouse_pos) <= cs;

                    note.hit_time.is_none() && !note.missed && within_hit_range
                },
                HitObject::Slider(slider) => {
                    let slider_ball_pos = coords.to_window(
                        slider.curve.position_at_time(state.time)
                    );
                    let distance = slider_ball_pos.distance_squared(mouse_pos);
                    let radius = CIRCLE_RADIUS_BASE * coords.cs;

                    let within_hit_range = distance <= (radius * OK_RADIUS_MULT).powi(2);

                    within_hit_range
                },
                HitObject::Spinner(_) => true,
            };

            if hit {
                let Some(key) = self.find_free_key() else { return };

                match note {
                    HitObject::Circle(_) => {
                        *self.key_states.get_mut(&key).unwrap() = KeyState::Tapped(state.time);
                        state.add_replay_action(ReplayAction::Press(key));
                    }
                    HitObject::Slider(_) | HitObject::Spinner(_) => {
                        *self.key_states.get_mut(&key).unwrap() = KeyState::Held(note_index);
                        state.add_replay_action(ReplayAction::Press(key));
                    }
                }
            }
        }

        if state.time >= note_end_time {
            match note {
                HitObject::Circle(_) => {}, // will get released in update
                HitObject::Slider(_) | HitObject::Spinner(_) => {
                    for (key, key_state) in self.key_states.iter_mut() {
                        let KeyState::Held(index) = key_state else { continue };
                        if *index != note_index { continue }
                        *key_state = KeyState::Unpressed;
                        state.add_replay_action(ReplayAction::Release(*key));
                        break;
                    }
                },
            }
        }
    }

    pub fn update(&mut self, time: f32) {
        for state in self.key_states.values_mut() {
            let KeyState::Tapped(pressed_time) = state else { continue };

            if time > *pressed_time + PRESS_DURATION {
                *state = KeyState::Unpressed;
            }
        }
    }

    pub fn key_pressed(&mut self, key: KeyPress) {
        let Some(state) = self.key_states.get_mut(&key) else { return };
        *state = KeyState::Manual;
    }
    pub fn key_released(&mut self, key: KeyPress) {
        let Some(state) = self.key_states.get_mut(&key) else { return };
        if *state == KeyState::Manual {
            *state = KeyState::Unpressed;
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum KeyState {
    /// key is not pressed
    #[default] Unpressed,

    /// key is pressed by us (RelaxPlay) for a note
    Tapped(f32),

    /// key is pressed by us (RelaxPlay) for a slider
    Held(usize),

    /// key is pressed by user
    Manual,
}
impl KeyState {
    fn is_free(&self) -> bool {
        let &Self::Unpressed = self else { return false };
        true
    }
}
