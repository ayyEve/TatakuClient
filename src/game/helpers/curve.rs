use crate::prelude::*;
use crate::beatmaps::osu::hitobject_defs::{CurveType, SliderDef};

#[derive(Copy, Clone, Debug)]
pub struct CurveLine {
    pub p1: Vector2,
    pub p2: Vector2,

    pub straight: bool,
    pub force_end: bool
}
impl CurveLine {
    pub fn new(p1: Vector2, p2: Vector2) -> Self {
        Self {
            p1,
            p2,
            straight: false,
            force_end: false
        }
    }

    pub fn rho(&self) -> f32 {
        length(self.p2 - self.p1)
    }
}


fn length(p:Vector2) -> f32 {
    let num = p.x * p.x + p.y*p.y;
    num.sqrt() as f32
}

#[derive(Clone, Debug)]
pub struct Curve {
    pub slider: SliderDef,
    pub path: Vec<CurveLine>,
    pub end_time: f32,

    pub smooth_lines: Vec<CurveLine>,
    pub cumulative_lengths: Vec<f32>,

    pub velocity: f32,
    pub score_times: Vec<f32>
}
impl Curve {
    fn new(slider: SliderDef, path: Vec<CurveLine>, beatmap: &Beatmap) -> Self {
        let l = slider.length * 1.4 * slider.slides as f32;
        let v2 = 100.0 * beatmap.get_beatmap_meta().slider_multiplier * 1.4;
        // let l = slider.length * slider.slides as f32;
        // let v2 = 100.0 * beatmap.metadata.slider_multiplier;
        let bl = beatmap.beat_length_at(slider.time, true);
        let end_time = slider.time + (l / v2 * bl) - 1.0;

        let velocity = beatmap.slider_velocity_at(slider.time);
        Self {
            path,
            slider,
            velocity,
            end_time,
            smooth_lines: Vec::new(),
            cumulative_lengths: Vec::new(),
            score_times: Vec::new()
        }
    }

    pub fn time_at_length(&self, length:f32) -> f32 {
        self.slider.time + (length / self.velocity) * 1000.0
    }

    pub fn length(&self) -> f32 {
        self.end_time - self.slider.time
    }

    pub fn get_non_normalized_length_required(&self, time: f32) -> f32 {
        let pos = (time - self.slider.time) / (self.length() / self.slider.slides as f32);
        self.cumulative_lengths.last().unwrap() * pos
    }
    
    pub fn get_length_required(&self, time: f32) -> f32 {
        let mut pos = (time - self.slider.time) / (self.length() / self.slider.slides as f32);
        if pos % 2.0 > 1.0 {
            pos = 1.0 - (pos % 1.0);
        } else {
            pos = pos % 1.0;
        }

        self.cumulative_lengths.last().unwrap() * pos
    }
    
    pub fn position_at_time(&self, time:f32) -> Vector2 {
        // if (this.sliderCurveSmoothLines == null) this.UpdateCalculations();
        if self.cumulative_lengths.len() == 0 {return self.slider.pos}
        if time < self.slider.time {return self.slider.pos}
        if time > self.end_time {return self.position_at_length(self.length())}

        // if (this.sliderCurveSmoothLines == null) this.UpdateCalculations();

        self.position_at_length(self.get_length_required(time))
    }

    pub fn position_at_length(&self, length:f32) -> Vector2 {
        // if (this.sliderCurveSmoothLines == null || this.cumulativeLengths == null) this.UpdateCalculations();
        if self.smooth_lines.len() == 0 || self.cumulative_lengths.len() == 0 {return self.slider.pos}
        
        if length == 0.0 {return self.smooth_lines[0].p1}
        
        let end = *self.cumulative_lengths.last().unwrap();

        if length > end {
            let end = self.smooth_lines.len();
            return self.smooth_lines[end - 1].p2;
        }
        let i = match self.cumulative_lengths.binary_search_by(|f| f.partial_cmp(&length).unwrap_or(std::cmp::Ordering::Greater)) {
            Ok(n) => n,
            Err(n) => n.min(self.cumulative_lengths.len() - 1),
        };

        let length_next = self.cumulative_lengths[i];
        let length_previous = if i == 0 {0.0} else {self.cumulative_lengths[i - 1]};
        
        let mut res = self.smooth_lines[i].p1;
    
        if length_next != length_previous {
            let n = (self.smooth_lines[i].p2 - self.smooth_lines[i].p1) 
                * ((length - length_previous) / (length_next - length_previous)) as f64;
            res = res + n;
        }

        res
    }
}


pub fn get_curve(slider:&SliderDef, beatmap: &Beatmap) -> Curve {
    let mut points = slider.curve_points.clone();
    points.insert(0, slider.pos);
    let mut path = Vec::new();

    let metadata = beatmap.get_beatmap_meta();

    match slider.curve_type {
        CurveType::Catmull => {
            for j in 0..points.len() {
                let v1 = if j >= 1 {points[j-1]} else {points[j]};
                let v2 = points[j];
                let v3 = if j + 1 < points.len() {points[j + 1]} else {v2 + (v2 - v1)};
                let v4 = if j + 2 < points.len() {points[j + 2]} else {v3 + (v3 - v2)};

                for k in 0..SLIDER_DETAIL_LEVEL {
                    path.push(CurveLine::new(
                        catmull_rom(v1,v2,v3,v4, k as f64 / SLIDER_DETAIL_LEVEL as f64),
                        catmull_rom(v1,v2,v3,v4, (k + 1) as f64 / SLIDER_DETAIL_LEVEL as f64)
                    ));
                }
                // path[path.Count - 1].forceEnd = true;
            }
        }
        CurveType::Bézier => {
            let mut last_index = 0;
            let mut i = 0;
            while i < points.len() {
                if metadata.beatmap_version > 8 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];
                    // println!("i: {}, p.len(): {}", i, points.len());

                    if multipart_segment || i == points.len() - 1 {
                        let this_length = points[last_index..i+1].to_vec();
                        if this_length.len() == 2 {
                            //we can use linear algorithm for this segment
                            let l = CurveLine::new(this_length[0], this_length[1]);
                            let segments = 1;
                            for j in 0..segments {
                                let mut line = CurveLine::new(
                                    l.p1 + (l.p2 - l.p1) * (j as f64 / segments as f64),
                                    l.p1 + (l.p2 - l.p1) * ((j + 1) as f64 / segments as f64)
                                );
                                line.straight = true;
                                path.push(line);
                            }
                        } else {
                            if metadata.beatmap_version < 10 {
                                //use the WRONG bezier algorithm. sliders will be 1/50 too short!
                                let points = create_bezier_wrong(this_length);
                                for j in 1..points.len() {
                                    path.push(CurveLine::new(points[j - 1], points[j]));
                                }
                            } else {
                                //use the bezier algorithm
                                let points = create_bezier(this_length);
                                for j in 1..points.len() {
                                    path.push(CurveLine::new(points[j - 1], points[j]));
                                }
                            }
                        }
                        // let len = path.len();
                        // if len > 1 {
                        //     path[len - 1].force_end = true;
                        // }

                        //Need to skip one point since we consuned an extra.
                        if multipart_segment {i += 1}
                        last_index = i;
                    }
                    

                } else if metadata.beatmap_version > 6 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];

                    if multipart_segment || i == points.len() - 1 {
                        let this_length = points[last_index..i + 1].to_vec();
                        let points = create_bezier_old(this_length);
                        
                        for j in 1..points.len() {
                            path.push(CurveLine::new(points[j-1], points[j]));
                        }
                        let len = path.len();
                        path[len - 1].force_end = true;
                        //Need to skip one point since we consuned an extra.
                        if multipart_segment {i += 1}
                        last_index = i;
                    }
                } else {
                    //This algorithm is broken for multipart sliders (http://osu.sifterapp.com/projects/4151/issues/145).
                    //Newer maps always use the one in the else clause.
                    if (i > 0 && points[i] == points[i - 1]) || i == points.len() - 1 {
                        let this_length = points[last_index..i + 1].to_vec();
                        let points = create_bezier_wrong(this_length);

                        for j in 1..points.len() {
                            path.push(CurveLine::new(points[j-1], points[j]));
                        }
                        
                        let len = path.len();
                        path[len - 1].force_end = true;
                        last_index = i;
                    }
                }
            
                i += 1;
            }
        }
        CurveType::Perfect => {
            // we may have 2 points when building the circle.
            if points.len() < 3 {
                let mut slider = slider.clone();
                slider.curve_type = CurveType::Linear;
                return get_curve(&slider, beatmap);
            }
            // more than 3 -> ignore them.
            if points.len() > 3 {
                let mut slider = slider.clone();
                slider.curve_type = CurveType::Bézier;
                return get_curve(&slider, beatmap);
            }
            let a = points[0];
            let b = points[1];
            let c = points[2];
            
            // all 3 points are on a straight line, avoid undefined behaviour:
            if is_straight_line(a,b,c) {
                let mut slider = slider.clone();
                slider.curve_type = CurveType::Linear;
                return get_curve(&slider, beatmap);
            }

            let (center, radius, t_initial, t_final) = circle_through_points(a,b,c);

            // this.curveLength = Math.Abs((t_final - t_initial) * radius);
            let curve_length = ((t_final - t_initial) * radius).abs();
            let segments = (curve_length * 0.125) as u32;
            let mut last_point = a;

            for i in 0..segments {
                let progress = i as f64 / segments as f64;
                let t = t_final * progress + t_initial * (1.0 - progress);
                let new_point = circle_point(center, radius, t);
                path.push(CurveLine::new(last_point, new_point));
                last_point = new_point;
            }

            path.push(CurveLine::new(last_point, c));
        }
        CurveType::Linear => {
            for i in 1..points.len() {
                let l = CurveLine::new(points[i - 1], points[i]);
                let segments = 1;

                for j in 0..segments {
                    let mut l2 = CurveLine::new(
                    l.p1 + (l.p2 - l.p1) * (j as f64 / segments as f64),
                    l.p1 + (l.p2 - l.p1) * ((j + 1) as f64 / segments as f64)
                    );
                    l2.straight = true;
                    path.push(l2);
                }
                let len = path.len();
                path[len - 1].force_end = true;
            }
        }
    }
    


    let mut curve = Curve::new(slider.clone(), path, beatmap);

    let path_count = curve.path.len();
    let mut total = 0.0;
    if path_count > 0 {
        //fill the cache
        curve.smooth_lines = curve.path.clone();
        curve.cumulative_lengths.clear();

        for l in 0..curve.path.len() {
            let mut add = curve.path[l].rho();
            if add.is_nan() {add = 0.0}
            total += add;
            curve.cumulative_lengths.push(total);
        }
    }

    if path_count < 1 {return curve}

    let ms_between_ticks = beatmap.beat_length_at(curve.slider.time, false) / metadata.slider_tick_rate;
    let mut t = curve.slider.time + ms_between_ticks;
    while t < curve.end_time {
        curve.score_times.push(t);
        t += ms_between_ticks;
    }
    
    curve
}

fn catmull_rom(value1:Vector2, value2:Vector2, value3:Vector2, value4:Vector2, amount:f64) -> Vector2 {
    let num = amount * amount;
    let num2 = amount * num;
    let mut result = Vector2::zero();

    result.x = 0.5 * (2.0 * value2.x + (-value1.x + value3.x) * amount + (2.0 * value1.x - 5.0 * value2.x + 4.0 * value3.x - value4.x) * num +
        (-value1.x + 3.0 * value2.x - 3.0 * value3.x + value4.x) * num2);

    result.y = 0.5 * (2.0 * value2.y + (-value1.y + value3.y) * amount + (2.0 * value1.y - 5.0 * value2.y + 4.0 * value3.y - value4.y) * num +
        (-value1.y + 3.0 * value2.y - 3.0 * value3.y + value4.y) * num2);
    return result;
}
