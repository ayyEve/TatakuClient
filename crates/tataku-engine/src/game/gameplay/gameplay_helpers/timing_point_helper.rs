use crate::prelude::*;

#[derive(Default)]
pub struct TimingPointHelper {
    timing_point_index: usize,
    control_point_index: usize,
    timing_points: Vec<TimingPoint>,
    pub slider_velocity_base: f32,

    next_beat: f32,
}
impl TimingPointHelper {
    pub fn timing_point(&self) -> &TimingPoint { self.indexed(self.timing_point_index) }
    pub fn control_point(&self) -> &TimingPoint { self.indexed(self.control_point_index) }
    pub fn next_beat(&self) -> f32 { self.next_beat }

    fn indexed(&self, index: usize) -> &TimingPoint { &self.timing_points[index % self.timing_points.len()] }

    pub fn new(mut timing_points: Vec<TimingPoint>, slider_velocity: f32) -> Self {
        // make sure timing_points are sorted
        timing_points.sort_by(|t,t2|t.time.partial_cmp(&t2.time).unwrap_or(core::cmp::Ordering::Equal));
        let (control_point_index, control_point) = timing_points.iter().enumerate().find(|(_,t)|!t.is_inherited()).unwrap();

        Self {
            timing_point_index: 0,
            control_point_index,
            next_beat: control_point.time,
            slider_velocity_base: slider_velocity,

            timing_points,
        }
    }
    pub fn update(&mut self, time: f32) -> Vec<TimingPointUpdate> {
        let mut update = Vec::with_capacity(2);
         
        if self.timing_point_index + 1 < self.timing_points.len() && self.timing_points[self.timing_point_index + 1].time <= time {
            let old_kiai = self.timing_point().kiai;

            self.timing_point_index += 1;
            let tp = *self.timing_point();
            if !tp.is_inherited() { 
                self.control_point_index = self.timing_point_index; 
                self.next_beat = tp.time;
            }

            if tp.kiai != old_kiai { update.push(TimingPointUpdate::KiaiChanged(tp.kiai)); }
        }

        if time >= self.next_beat {
            let beat_length = self.control_point().beat_length;
            let measure_length = beat_length * self.control_point().meter as f32;

            // gonna use half a measure for now but i'm not sure if this is correct
            let pulse_length = measure_length / 2.0;
            self.next_beat += pulse_length;
            update.push(TimingPointUpdate::BeatHappened(pulse_length));
        }

        update
    }
    pub fn reset(&mut self) {
        self.timing_point_index = 0;
        let (control_point_index, control_point) = self.timing_points.iter().enumerate().find(|(_,t)|!t.is_inherited()).unwrap();
        self.control_point_index = control_point_index;
        self.next_beat = control_point.time;
    }

    
    pub fn timing_point_at(&self, time: f32, allow_inherited: bool) -> &TimingPoint {
        let mut tp = &self.timing_points[0];

        for i in self.timing_points.iter() {
            if i.is_inherited() && !allow_inherited { continue }
            if i.time <= time { tp = i }
        }

        tp
    }



    // moved here from the beatmap object because its annoying having things in multiple places
    pub fn beat_length_at(&self, time:f32, allow_multiplier:bool) -> f32 {
        if self.timing_points.is_empty() { return 0.0 }

        // this isnt always a control point, need to find the first non-inherited point
        let mut point = self.timing_points.iter().find(|t|!t.is_inherited());
        let mut inherited_point = None;

        for tp in self.timing_points.iter() {
            if tp.time <= time {
                if tp.is_inherited() {
                    inherited_point = Some(tp);
                } else {
                    point = Some(tp);
                }
            }
        }

        let mut mult = 1.0;
        let Some(p) = point else { return 0.0 };

        if let Some(ip) = inherited_point.filter(|_| allow_multiplier) {
            if p.time <= ip.time && ip.beat_length < 0.0 {
                mult = (-ip.beat_length).clamp(10.0, 1000.0) / 100.0;
            }
        }

        p.beat_length * mult
    }
    pub fn slider_velocity_at(&self, time:f32) -> f32 {
        let bl = self.beat_length_at(time, true);
        100.0 * (self.slider_velocity_base * 1.4) * if bl > 0.0 {1000.0 / bl} else {1.0}
    }
    pub fn control_point_at(&self, time:f32) -> TimingPoint {
        // panic as this should be dealt with earlier in the code
        if self.timing_points.is_empty() { panic!("beatmap has no timing points!"); }

        let mut point = self.timing_points[0];
        for tp in self.timing_points.iter() {
            if tp.time <= time {point = *tp}
        }

        point
    }


}

impl Deref for TimingPointHelper {
    type Target = Vec<TimingPoint>;
    
    fn deref(&self) -> &Self::Target {
        &self.timing_points
    }
}

pub enum TimingPointUpdate {
    KiaiChanged(bool),
    BeatHappened(f32),
}
