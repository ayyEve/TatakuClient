
use prelude::taiko::TaikoHitObject;

use crate::prelude::*;

// constants
pub const DECAY_BASE:f64 = 0.30;
const COLOR_CHANGE_BONUS:f64 = 0.75;
const RHYTHM_CHANGE_BONUS:f64 = 1.0;
const RHYTHM_CHANGE_BASE_THRESHOLD:f64 = 0.2;
const RHYTHM_CHANGE_BASE:f64 = 2.0;

#[derive(Clone)]
pub struct DifficultyHitObject {
    pub strain: f64, // 1 default
    same_color_since: u32, // 1 default
    last_color_switch_even: ColorSwitch,
    time_elapsed: f64,

    pub time: f32,
    pub note_type: NoteType,
    pub is_kat: bool,
}

impl DifficultyHitObject {
    pub fn new(base:&Box<dyn TaikoHitObject>) -> Self {
        Self {
            same_color_since: 1,
            strain: 1.0,
            last_color_switch_even: ColorSwitch::None,
            time_elapsed: 0.0,

            time: base.time(),
            note_type: base.note_type(),
            is_kat: base.is_kat()
        }
    }

    // pub fn calculate_strains(&mut self, previous:&DifficultyHitObject, time_rate:f64) {
    //     self.time_elapsed = (self.time - previous.time) / time_rate;
    //     let decay = DECAY_BASE.powf(self.time_elapsed / 1000.0);

    //     let mut addition = 1.0;
    //     if previous.note_type == NoteType::Note && self.note_type == NoteType::Note &&
    //         (self.time - previous.time) < 1000.0 {

    //         // color change addition
    //         if previous.is_kat != self.is_kat {
    //             self.last_color_switch_even = if previous.same_color_since % 2 == 0 {ColorSwitch::Even} else {ColorSwitch::Odd};
    //             if previous.last_color_switch_even != ColorSwitch::None && previous.last_color_switch_even != self.last_color_switch_even {
    //                 addition += COLOR_CHANGE_BONUS;
    //             }
    //         } else {
    //             self.last_color_switch_even = previous.last_color_switch_even;
    //             self.same_color_since = previous.same_color_since + 1;
    //         }

    //         // rhythm change addition
    //         // We don't want a division by zero if some random mapper decides to put 2 HitObjects at the same time.
    //         if !(self.time_elapsed == 0.0 || previous.time_elapsed == 0.0) {
    //             let time_elapsed_ratio = (previous.time_elapsed / self.time_elapsed).max(self.time_elapsed / previous.time_elapsed);
    //             if !(time_elapsed_ratio >= 8.0) {
    //                 let difference = time_elapsed_ratio.log(RHYTHM_CHANGE_BASE) % 1.0;
    //                 if difference > RHYTHM_CHANGE_BASE_THRESHOLD && difference < 1.0 - RHYTHM_CHANGE_BASE_THRESHOLD {
    //                     addition += RHYTHM_CHANGE_BONUS;
    //                 }
    //             }
    //         }
    //     }

    //     let mut addition_factor = 1.0;
    //     if self.time_elapsed < 50.0 {
    //         addition_factor = 0.4 + 0.6 * self.time_elapsed / 50.0;
    //     }

    //     self.strain = previous.strain * decay + addition * addition_factor;
    // }

}

#[derive(Clone, Copy, PartialEq)]
enum ColorSwitch {
    None = 0,
    Even,
    Odd
}