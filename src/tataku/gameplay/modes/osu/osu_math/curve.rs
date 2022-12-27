use crate::prelude::*;
use super::super::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct CurveLine {
    pub p1: Vector2,
    pub p2: Vector2,
}
impl CurveLine {
    pub fn new(p1: Vector2, p2: Vector2) -> Self {
        Self {
            p1,
            p2,
        }
    }

    pub fn rho(&self) -> f32 {
        let p = self.p2 - self.p1;
        (p.x*p.x + p.y*p.y).sqrt() as f32
    }
}

#[derive(Clone, Debug)]
pub struct Curve {
    pub slider: SliderDef,
    pub end_time: f32,

    pub segments: Vec<CurveSegment>,
    pub curve_lines: Vec<CurveLine>,
    pub lengths: Vec<f32>,

    pub velocity: f32,
    pub score_times: Vec<f32>
}
#[allow(dead_code)]
impl Curve {
    fn new(slider: SliderDef, path: Vec<CurveSegment>, beatmap: &Beatmap) -> Self {
        let mut slider_multiplier = 1.0;
        if let Beatmap::Osu(map) = beatmap {
            slider_multiplier = map.slider_multiplier;
        }


        let l = slider.length * 1.4 * slider.slides as f32;
        let v2 = 100.0 * slider_multiplier * 1.4;
        // let l = slider.length * slider.slides as f32;
        // let v2 = 100.0 * beatmap.metadata.slider_multiplier;
        let bl = beatmap.beat_length_at(slider.time, true);
        let end_time = slider.time + (l / v2 * bl) - 1.0;

        let velocity = beatmap.slider_velocity_at(slider.time);
        Self {
            segments: path,
            slider,
            velocity,
            end_time,
            curve_lines: Vec::new(),
            lengths: Vec::new(),
            score_times: Vec::new()
        }
    }

    pub fn time_at_length(&self, length:f32) -> f32 {
        self.slider.time + (length / self.velocity) * 1000.0
    }

    pub fn length(&self) -> f32 {
        self.end_time - self.slider.time
    }
    
    pub fn get_length_required(&self, time: f32) -> f32 {
        let mut pos = (time - self.slider.time) / (self.length() / self.slider.slides as f32);
        if pos % 2.0 > 1.0 {
            pos = 1.0 - (pos % 1.0);
        } else {
            pos = pos % 1.0;
        }

        self.lengths.last().unwrap() * pos
    }
    
    pub fn position_at_time(&self, time:f32) -> Vector2 {
        // if (this.sliderCurveSmoothLines == null) this.UpdateCalculations();
        if self.lengths.len() == 0 { return self.slider.pos }
        if time < self.slider.time { return self.slider.pos }
        if time > self.end_time { return self.position_at_length(self.length()) }

        // if (this.sliderCurveSmoothLines == null) this.UpdateCalculations();

        self.position_at_length(self.get_length_required(time))
    }

    pub fn position_at_length(&self, length:f32) -> Vector2 {
        // if (this.sliderCurveSmoothLines == null || this.cumulativeLengths == null) this.UpdateCalculations();
        if self.curve_lines.len() == 0 || self.lengths.len() == 0 {return self.slider.pos}
        
        if length == 0.0 {return self.curve_lines[0].p1}
        
        let end = *self.lengths.last().unwrap();

        if length > end {
            let end = self.curve_lines.len();
            return self.curve_lines[end - 1].p2;
        }
        let i = match self.lengths.binary_search_by(|f| f.partial_cmp(&length).unwrap_or(std::cmp::Ordering::Greater)) {
            Ok(n) => n,
            Err(n) => n.min(self.lengths.len() - 1),
        };

        let length_next = self.lengths[i];
        let length_previous = if i == 0 {0.0} else {self.lengths[i - 1]};
        
        let mut res = self.curve_lines[i].p1;
    
        if length_next != length_previous {
            let n = (self.curve_lines[i].p2 - self.curve_lines[i].p1) 
                * ((length - length_previous) / (length_next - length_previous)) as f64;
            res = res + n;
        }

        res
    }
}


#[derive(Clone, Debug)]
pub enum CurveSegment {
    Bezier {
        curve: Vec<Vector2>, 
        // control_points: Vec<Vector2>
    },

    Linear {
        p1: Vector2, 
        p2: Vector2
    },

    Catmull {
        curve: Vec<Vector2>
    },

    Perfect {
        curve: Vec<Vector2>
    },
}



pub fn get_curve(slider:&SliderDef, beatmap: &Beatmap) -> Curve {
    let mut points = slider.curve_points.clone();
    points.insert(0, slider.pos);

    let mut path = Vec::new();

    // let metadata = beatmap.get_beatmap_meta();

    let mut beatmap_version = 10;
    let mut slider_tick_rate = 1.0;
    if let Beatmap::Osu(map) = beatmap {
        beatmap_version = map.beatmap_version;
        slider_tick_rate = map.slider_tick_rate;
    }


    match slider.curve_type {
        CurveType::Catmull => {
            for j in 0..points.len() {
                let v1 = if j >= 1 {points[j-1]} else {points[j]};
                let v2 = points[j];
                let v3 = if j + 1 < points.len() {points[j + 1]} else {v2 + (v2 - v1)};
                let v4 = if j + 2 < points.len() {points[j + 2]} else {v3 + (v3 - v2)};

                let mut curve = Vec::new();
                for k in 0..=SLIDER_DETAIL_LEVEL {
                    curve.push(catmull_rom(v1, v2, v3, v4, k as f64 / SLIDER_DETAIL_LEVEL as f64));
                }
                path.push(CurveSegment::Catmull { curve })
            }
        }
        CurveType::Bézier => {
            let mut last_index = 0;
            let mut i = 0;
            while i < points.len() {
                if beatmap_version > 6 {
                    let multipart_segment = (i as i32) < points.len() as i32 - 2 && points[i] == points[i + 1];

                    if multipart_segment || i == points.len() - 1 {
                        let this_segment = points[last_index..i+1].to_vec();

                        if beatmap_version > 8 {
                            if this_segment.len() == 2 {
                                // this segment is a line
                                path.push(CurveSegment::Linear {p1: this_segment[0], p2:this_segment[1]});
                            } else {
                                let curve = create_bezier(this_segment, beatmap_version < 10);
                                path.push(CurveSegment::Bezier {curve});
                            }
                        } else {
                            let this_length = points[last_index..i + 1].to_vec();
                            let curve = create_bezier(this_length, false);
                            path.push(CurveSegment::Bezier {curve});
                        }

                        //Need to skip one point since we consuned an extra.
                        if multipart_segment {i += 1}
                        last_index = i;
                    }

                } else {
                    //This algorithm is broken for multipart sliders (http://osu.sifterapp.com/projects/4151/issues/145).
                    //Newer maps always use the one in the else clause.
                    if (i > 0 && points[i] == points[i - 1]) || i == points.len() - 1 {
                        let this_segment = points[last_index..i + 1].to_vec();
                        let curve = create_bezier(this_segment, true);
                        path.push(CurveSegment::Bezier {curve});
                        
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

            let mut curve = Vec::new();
            curve.push(a);

            for i in 0..segments {
                let progress = i as f64 / segments as f64;
                let t = t_final * progress + t_initial * (1.0 - progress);
                let new_point = circle_point(center, radius, t);
                curve.push(new_point);
            }

            path.push(CurveSegment::Perfect { curve });

        }
        CurveType::Linear => {
            for i in 1..points.len() {
                path.push(CurveSegment::Linear { p1: points[i - 1], p2:points[i] });
            }
        }
    }
    

    let mut smooth_path = Vec::new();
    for i in path.iter() {
        match i {
            CurveSegment::Bezier { curve }
            | CurveSegment::Catmull { curve }
            | CurveSegment::Perfect { curve } => {
                for i in 1..curve.len() {
                    let p1 = curve[i - 1];
                    let p2 = curve[i];
                    smooth_path.push(CurveLine::new(p1, p2))
                }
            },

            CurveSegment::Linear { p1, p2 } => smooth_path.push(CurveLine::new(*p1, *p2)),
        }
    }


    let mut curve = Curve::new(slider.clone(), path, beatmap);

    let path_count = curve.segments.len();
    let mut total = 0.0;
    if path_count > 0 {
        //fill the cache
        curve.curve_lines = smooth_path;
        curve.lengths.clear();

        for l in 0..curve.curve_lines.len() {
            let mut add = curve.curve_lines[l].rho();
            if add.is_nan() { add = 0.0 }
            total += add;
            curve.lengths.push(total);
        }
    }

    if path_count < 1 {return curve}

    let ms_between_ticks = beatmap.beat_length_at(curve.slider.time, false) / slider_tick_rate;
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

