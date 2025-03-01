use crate::prelude::*;

// const KEYS: [KeyPress; 4] = [
//     KeyPress::Left,
//     KeyPress::Right,
//     KeyPress::LeftMouse,
//     KeyPress::RightMouse,
// ];

pub struct StandardAutoHelper {
    // point_trail_angle: Vector2,
    point_trail_start_time: f32,
    point_trail_end_time: f32,
    point_trail_start_pos: Vector2,
    point_trail_end_pos: Vector2,

    /// list of notes currently being held, and what key was held for that note
    holding: HashMap<usize, KeyPress>,

    release_queue: Vec<ReplayAction>,

    press_counter: usize,
}
impl StandardAutoHelper {
    pub fn new() -> Self {
        Self {
            // point_trail_angle: Vector2::ZERO,
            point_trail_start_time: 0.0,
            point_trail_end_time: 0.0,
            point_trail_start_pos: Vector2::ZERO,
            point_trail_end_pos: Vector2::ZERO,

            holding: HashMap::new(),

            release_queue: Vec::new(),
            press_counter: 0
        }
    }
    pub fn get_release_queue(&mut self) -> Vec<ReplayAction> {
        std::mem::take(&mut self.release_queue)
    }

    pub fn get_key(&self) -> KeyPress {
        if self.press_counter % 2 == 0 {
            KeyPress::LeftMouse
        } else {
            KeyPress::RightMouse
        }
    }

    pub fn update(
        &mut self, 
        time: f32, 
        notes: &[Box<dyn OsuHitObject>], 
        scaling_helper: &Arc<ScalingHelper>, 
        frames: &mut Vec<ReplayAction>
    ) {
        let mut any_checked = false;

        let map_over = time > notes.last().map(|n| n.end_time(100.0)).unwrap_or(0.0);
        if map_over { return; }

        for i in 0..notes.len() {
            let note = &notes[i];
            if note.was_hit() { continue }

            if self.holding.contains_key(&i) {
                if time >= note.end_time(0.0) {
                    let k = self.holding.remove(&i).unwrap_or(KeyPress::LeftMouse);
                    self.release_queue.push(ReplayAction::Release(k));

                    let pos = scaling_helper.descale_coords(note.pos_at(time));
                    if i+1 >= notes.len() {
                        self.point_trail_start_pos = pos;
                        self.point_trail_end_pos = pos;
                        continue;
                    }
                    
                    let next_note = &notes[i + 1];

                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(next_note.pos_at(self.point_trail_end_time));
                    
                    self.point_trail_start_time = time;
                    self.point_trail_end_time = next_note.time();
                } else {
                    let pos = scaling_helper.descale_coords(note.pos_at(time));
                    // move the mouse to the pos
                    frames.push(ReplayAction::MousePos(pos.x, pos.y));
                }
                
                any_checked = true;
                continue;
            }
            
            if time >= note.time() {
                let pos = scaling_helper.descale_coords(note.pos_at(time));
                // move the mouse to the pos
                frames.push(ReplayAction::MousePos(pos.x, pos.y));
                
                self.press_counter += 1;
                let k = self.get_key();
                frames.push(ReplayAction::Press(k));
                if note.note_type() == NoteType::Note {
                    // TODO: add some delay to the release?
                    self.release_queue.push(ReplayAction::Release(k));
                } else {
                    self.holding.insert(i, k);
                }

                // if this was the last note
                if i + 1 >= notes.len() {
                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(scaling_helper.window_size / 2.0);
                    
                    self.point_trail_start_time = 0.0;
                    self.point_trail_end_time = 1.0;
                    continue;
                }

                // draw a line to the next note
                let next_note = &notes[i + 1];

                self.point_trail_start_pos = pos;
                self.point_trail_end_pos = scaling_helper.descale_coords(next_note.pos_at(self.point_trail_end_time));
                
                self.point_trail_start_time = time;
                self.point_trail_end_time = next_note.time();

                any_checked = true;
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

    pub fn time_skip(
        &mut self, 
        new_time: f32, 
        notes: &[Box<dyn OsuHitObject>], 
        scaling_helper: &Arc<ScalingHelper>, 
        frames: &mut Vec<ReplayAction>
    ) {
        let map_over = new_time > notes
            .last()
            .map(|n| n.end_time(100.0))
            .unwrap_or(0.0);
        if map_over { return; }

        let mut mouse_pos = None;
        // let mut presses = HashSet::new();
        // let mut releases = HashSet::new();

        for i in 0..notes.len() {
            let note = &notes[i];
            if note.was_hit() { continue }

            if self.holding.contains_key(&i) {
                if new_time >= note.end_time(0.0) {
                    // let k = self.holding
                    //     .remove(&i)
                    //     .unwrap_or(KeyPress::LeftMouse);
                    // self.release_queue.push(ReplayAction::Release(k));
                    // releases.insert(&k);

                    let pos = scaling_helper.descale_coords(note.pos_at(new_time));
                    if i + 1 >= notes.len() {
                        self.point_trail_start_pos = pos;
                        self.point_trail_end_pos = pos;
                        continue;
                    }
                    
                    let next_note = &notes[i + 1];

                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(next_note.pos_at(self.point_trail_end_time));
                    
                    self.point_trail_start_time = new_time;
                    self.point_trail_end_time = next_note.time();
                } else {
                    let pos = scaling_helper.descale_coords(note.pos_at(new_time));
                    // move the mouse to the pos
                    mouse_pos = Some(pos);
                    // frames.push(ReplayAction::MousePos(pos.x, pos.y));
                }
                
                continue;
            }
            
            if new_time >= note.time() {
                let pos = scaling_helper.descale_coords(note.pos_at(new_time));
                // move the mouse to the pos
                mouse_pos = Some(pos);
                // frames.push(ReplayAction::MousePos(pos.x, pos.y));
                
                self.press_counter += 1;
                let k = self.get_key();
                // presses.insert(k);
                // frames.push(ReplayAction::Press(k));
                if note.note_type() == NoteType::Note {
                    // self.release_queue.push(ReplayAction::Release(k));
                    // releases.insert(&k);
                } else {
                    self.holding.insert(i, k);
                }

                // if this was the last note
                if i + 1 >= notes.len() {
                    self.point_trail_start_pos = pos;
                    self.point_trail_end_pos = scaling_helper.descale_coords(scaling_helper.window_size / 2.0);
                    
                    self.point_trail_start_time = 0.0;
                    self.point_trail_end_time = 1.0;
                    continue;
                }

                // draw a line to the next note
                let next_note = &notes[i + 1];

                self.point_trail_start_pos = pos;
                self.point_trail_end_pos = scaling_helper.descale_coords(next_note.pos_at(self.point_trail_end_time));
                
                self.point_trail_start_time = new_time;
                self.point_trail_end_time = next_note.time();
            }
        }


        if let Some(pos) = mouse_pos {
            frames.push(ReplayAction::MousePos(pos.x, pos.y));
        }
        // for i in KEYS {
        //     if presses.contains(&i) && !releases.contains(&i) {

        //     } else if releases.contains(&i) {

        //     }
        // }
    }
}
