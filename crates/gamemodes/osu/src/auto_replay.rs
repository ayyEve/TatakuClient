use crate::prelude::*;
use common::{
    replays::*,
};
use tataku::{
    Vector2,
};

// const KEYS: [KeyPress; 4] = [
//     KeyPress::Left,
//     KeyPress::Right,
//     KeyPress::LeftMouse,
//     KeyPress::RightMouse,
// ];

#[derive(Default)]
pub struct AutoReplay {
    point_trail_start_time: f32,
    point_trail_end_time: f32,
    point_trail_start_pos: Vector2,
    point_trail_end_pos: Vector2,

    /// list of notes currently being held, and what key was held for that note
    holding: HashMap<usize, KeyPress>,

    release_queue: Vec<ReplayAction>,

    press_counter: usize,
}
impl AutoReplay {
    pub fn get_release_queue(&mut self) -> Vec<ReplayAction> {
        std::mem::take(&mut self.release_queue)
    }

    pub fn get_key(&self) -> KeyPress {
        if self.press_counter.is_multiple_of(2) {
            KeyPress::LeftMouse
        } else {
            KeyPress::RightMouse
        }
    }

    pub fn update(
        &mut self,
        time: f32,
        notes: &[HitObject],
        notes_skip: usize,
        frames: &mut Vec<ReplayAction>
    ) {
        let mut any_checked = false;

        for i in notes_skip..notes.len() {
            let note = &notes[i];

            let move_to_next_note = match note {
                HitObject::Circle(note) => {
                    if note.hit_time.is_some() || note.missed { continue; }

                    if time >= note.time {
                        any_checked = true;

                        self.press_counter += 1;

                        let k = self.get_key();
                        frames.push(ReplayAction::Press(k));
                        // TODO: add some delay to the release?
                        self.release_queue.push(ReplayAction::Release(k));

                        // move the mouse to the pos
                        frames.push(ReplayAction::MousePos(note.pos.x, note.pos.y));

                        self.point_trail_start_pos = note.pos;
                        self.point_trail_start_time = time;

                        true
                    } else {
                        break;
                    }
                },
                HitObject::Slider(slider) => {
                    if time >= slider.curve.end_time {
                        if let Some(k) = self.holding.remove(&i) {
                            any_checked = true;

                            self.release_queue.push(ReplayAction::Release(k));

                            self.point_trail_start_pos = slider.time_end_pos;
                            self.point_trail_start_time = slider.curve.end_time;

                            true
                        } else {
                            continue;
                        }
                    } else if time >= slider.time {
                        any_checked = true;

                        let slider_ball_pos = slider.curve.position_at_time(time);

                        // move the mouse to the pos
                        frames.push(ReplayAction::MousePos(slider_ball_pos.x, slider_ball_pos.y));

                        if !self.holding.contains_key(&i) && slider.initial_hit.is_none() {
                            self.press_counter += 1;

                            let k = self.get_key();
                            frames.push(ReplayAction::Press(k));
                            self.holding.insert(i, k);
                        }

                        false
                    } else {
                        break;
                    }
                },
                HitObject::Spinner(spinner) => {
                    let center = FIELD_SIZE / 2.0;
                    let radius = FIELD_SIZE.x / 4.0;

                    if time >= spinner.end_time {
                        if let Some(k) = self.holding.remove(&i) {
                            any_checked = true;

                            self.release_queue.push(ReplayAction::Release(k));

                            self.point_trail_start_pos = center + Vector2::new(0.0, radius);
                            self.point_trail_start_time = spinner.end_time;

                            true
                        } else {
                            continue
                        }
                    } else if time >= spinner.time {
                        any_checked = true;

                        if !self.holding.contains_key(&i) && !spinner.judged {
                            self.press_counter += 1;

                            let k = self.get_key();
                            frames.push(ReplayAction::Press(k));
                            self.holding.insert(i, k);
                        }

                        let progress = (time - spinner.time) / (spinner.end_time - spinner.time);

                        let angle = TWO_PI * spinner.rotations_required as f32 * progress;

                        let offset = Vector2::new(
                            angle.sin(),
                            angle.cos(),
                        ) * radius;

                        let mouse_pos = center + offset;

                        // move the mouse to the pos
                        frames.push(ReplayAction::MousePos(mouse_pos.x, mouse_pos.y));

                        false
                    } else {
                        break;
                    }
                },
            };

            if move_to_next_note {
                let center = FIELD_SIZE / 2.0;

                if let Some(next_note) = notes.get(i + 1) {
                    let pos = match next_note {
                        HitObject::Circle(note) => note.pos,
                        HitObject::Slider(slider) => slider.pos,
                        HitObject::Spinner(_) => center,
                    };

                    self.point_trail_end_pos = pos;
                    self.point_trail_end_time = next_note.time();
                } else {
                    self.point_trail_end_pos = center;
                    self.point_trail_end_time = self.point_trail_start_time + 1000.0;
                }
            }
        }

        if any_checked { return }

        // if we got here no notes were updated
        // follow the point_trail
        let duration = self.point_trail_end_time - self.point_trail_start_time;
        let current = time - self.point_trail_start_time;
        let len = current / duration;

        let new_pos = Vector2::lerp(self.point_trail_start_pos, self.point_trail_end_pos, len.clamp(0.0, 1.0));
        frames.push(ReplayAction::MousePos(new_pos.x, new_pos.y));
    }
}
