use std::f32::consts::PI;
#[cfg(feature="graphics")]
use engine::graphics;

use crate::prelude::*;

use std::collections::VecDeque;

use tataku::{
    Color,
    Border,
    Vector2,
    ColorField,
};

use engine::{
    beatmaps::osu::*,
    gameplay::{
        judgments::HitJudgment,
        Hitsound,
    }
};


const SLIDER_DOT_RADIUS:f32 = 8.0;
const BORDER_RADIUS:f32 = 6.0;
const BORDER_COLOR:Color = Color::WHITE;
const BEAT_SCALE: f32 = 1.4;

pub const OK_RADIUS_MULT: f32 = 4.0;
const OK_TICK_RADIUS_MULT: f32 = 2.0;

#[derive(Default)]
pub struct Slider {
    /// curve that defines the slider
    pub curve: Curve,
    speed: f32,

    /// start pos
    pub pos: Vector2,
    /// visual end pos
    pub visual_end_pos: Vector2,
    /// time end pos
    pub time_end_pos: Vector2,

    /// hit dots. if the slider isnt being held for these
    #[cfg(feature="graphics")] hit_dots: VecDeque<SliderDot>,

    pub hitsound: u8,
    pub hitsamples: HitSamples,
    pub edge_sets: Vec<[u8; 2]>,
    pub edge_sounds: Vec<u8>,

    pub initial_hit: Option<HitJudgment>,
    pub pending_combo: Vec<(HitJudgment, Vector2)>,
    pub judgements_added: usize,

    /// start time
    pub time: f32,
    slides: u64,
    /// how many slides have been completed?
    slides_complete: u64,
    /// used to check if a slide has been completed
    moving_forward: bool,

    /// combo color
    color_index: usize,

    /// was a slider dot missed
    dots_missed: usize,
    /// how many dots is there
    dot_count: usize,

    /// is the mouse in a good state for sliding? (pos + key down)
    sliding_ok: bool,

    #[cfg(feature="graphics")] circle: HitCircle,
    #[cfg(feature="graphics")] pub approach_circle: ApproachCircle,

    #[cfg(feature="graphics")] end_circle_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] slider_reverse_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] sliderball_image: Option<graphics::Animation>,
    #[cfg(feature="graphics")] sliderball_under_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] follow_circle_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] dot_image: Option<graphics::Image>,

    #[cfg(feature="graphics")] slider_cs: f32,
    #[cfg(feature="graphics")] slider_body: Option<graphics::SliderDrawable>,
    #[cfg(feature="graphics")] bounds: tataku::Bounds,

    #[cfg(feature="graphics")] pub shake: Shake,
}
impl Slider {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        def: &SliderDef,
        curve: Curve,
        combo_num: u16,
        color_index: usize,
        speed: f32
    ) -> Self {
        let time = def.time;

        let pos = def.pos;
        let visual_end_pos = curve.curve_lines.last().unwrap().p2;
        let time_end_pos = if def.slides % 2 == 1 {visual_end_pos} else {pos};

        let slides = def.slides;

        let hitsound = def.hitsound;
        let hitsamples = def.hitsamples.clone();
        let edge_sets = def.edge_sets.clone();
        let edge_sounds = def.edge_sounds.clone();

        Self {
            curve,

            pos,
            visual_end_pos,
            time_end_pos,

            time,
            moving_forward: true,
            slides,

            hitsound,
            hitsamples,
            edge_sets,
            edge_sounds,

            color_index,

            #[cfg(feature="graphics")] circle: HitCircle::new(combo_num),
            #[cfg(feature="graphics")] approach_circle: ApproachCircle::new(),

            speed,

            ..Self::default()
        }
    }

    #[cfg(feature="graphics")]
    fn make_body(&mut self, cs: f32) {
        let radius = CIRCLE_RADIUS_BASE * cs;
        let border_radius = BORDER_RADIUS * cs;

        let mut min_pos = FIELD_SIZE;
        let mut max_pos = Vector2::ZERO;

        // FIXME: figure out why radius is 0?
        if radius <= 0.01 {
            return;
        }

        let mut line_segments: Vec<tataku::LineSegment> = self.curve.segments
            .iter()
            .flat_map(|segment| {
                let points = segment.all_points();

                if points.is_empty() { return Vec::new(); }
                if points.is_empty() { return Vec::new(); }

                // Calculate bounds of first point
                let first = *points.first().unwrap();
                min_pos.x = min_pos.x.min(first.x);
                min_pos.y = min_pos.y.min(first.y);
                max_pos.x = max_pos.x.max(first.x);
                max_pos.y = max_pos.y.max(first.y);

                points.windows(2).map(|points| {
                    let p1 = points[0];
                    let p2 = points[1];

                    // Calculate bounds of remaining points
                    min_pos.x = min_pos.x.min(p2.x);
                    min_pos.y = min_pos.y.min(p2.y);
                    max_pos.x = max_pos.x.max(p2.x);
                    max_pos.y = max_pos.y.max(p2.y);

                    tataku::LineSegment { p1, p2 }
                }).collect::<Vec<_>>() // todo: avoid too many allocations here
            }).collect();

        min_pos -= radius;
        max_pos += radius;

        let size = max_pos - min_pos;

        let border_width = border_radius;
        let circle_radius = radius - border_width;
        let cell_size = radius;

        let grid_size = size / cell_size;
        let grid_size = [1 + grid_size.x.floor() as u32, 1 + grid_size.y.floor() as u32];
        // let grid_size = [grid_size.x.ceil().max(1.0) as u32, grid_size.y.ceil().max(1.0) as u32];

        let mut grid_cells: Vec<Vec<u32>> = vec![Vec::new(); (grid_size[0] * grid_size[1]) as usize];

        line_segments.iter_mut().enumerate()
            .for_each(|(i, s)| {
                s.p1 -= min_pos;
                s.p2 -= min_pos;

                let grid_coord = s.p1 / cell_size;
                let (mut grid_x, mut grid_y) = (grid_coord.x.floor() as u32, grid_coord.y.floor() as u32);

                let p1_grid_index = grid_y * grid_size[0] + grid_x;
                if grid_cells.len() <= p1_grid_index as usize { return }
                grid_cells[p1_grid_index as usize].push(i as u32);

                let grid_coord_p2 = s.p2 / cell_size;
                let grid_coord_p2 = (grid_coord_p2.x.floor() as u32, grid_coord_p2.y.floor() as u32);

                // Traverse grid along line segment.

                let dir = (s.p2 - s.p1).normalize();

                if dir.length_squared() == 0.0 { return; }

                let dir_sign = (dir.x.signum() as i32, dir.y.signum() as i32);

                let grid_size_along_segment = Vector2::new(
                    cell_size / dir.x.abs(),
                    cell_size / dir.y.abs()
                );

                let pos_in_cell = Vector2::new(
                    s.p1.x - grid_x as f32 * cell_size,
                    s.p1.y - grid_y as f32 * cell_size,
                );

                let distance_next_cell = Vector2::new(
                    dir_sign.0.is_positive() as u32 as f32 * cell_size,
                    dir_sign.1.is_positive() as u32 as f32 * cell_size,
                ) - pos_in_cell;

                // this should always be >=0 but abs takes care of -0.
                let mut distance_along_segment_next_cell = (distance_next_cell / dir).abs();

                loop {
                    if distance_along_segment_next_cell.x < distance_along_segment_next_cell.y {
                        distance_along_segment_next_cell.x += grid_size_along_segment.x;

                        let Some(new) = grid_x.checked_add_signed(dir_sign.0) else { break; };
                        grid_x = new;
                    } else {
                        distance_along_segment_next_cell.y += grid_size_along_segment.y;

                        let Some(new) = grid_y.checked_add_signed(dir_sign.1) else { break; };
                        grid_y = new;
                    }

                    // Out of bounds
                    if  (dir_sign.0.is_positive() && (grid_x > grid_coord_p2.0 || grid_x > grid_size[0])) ||
                        (dir_sign.0.is_negative() && grid_x < grid_coord_p2.0) ||
                        (dir_sign.1.is_positive() && (grid_y > grid_coord_p2.1 || grid_y > grid_size[1])) ||
                        (dir_sign.1.is_negative() && grid_y < grid_coord_p2.1)
                    {
                        break;
                    }

                    let grid_index = grid_y * grid_size[0] + grid_x;
                    grid_cells[grid_index as usize].push(i as u32);
                }
            });

        let grid_cells_len = grid_cells.len();

        let (slider_grids, grid_cells) = grid_cells.into_iter()
            .fold(
                (Vec::with_capacity(grid_cells_len), Vec::with_capacity(grid_cells_len)),
                |(mut indexes, mut cells), v| {
                    indexes.push(tataku::GridCell {
                        index: cells.len() as u32,
                        length: v.len() as u32,
                    });
                    cells.extend(v);

                    (indexes, cells)
                }
            );

        let slider_data = tataku::SliderData {
            circle_radius,
            border_width,
            snake_percentage: 1.0,
            slider_velocity: self.speed,
            size,
            scale: 1.0,
            grid_size,
            grid_index: 0,
            body_color: Color::GREY,
            border_color: Color::WHITE,
        };

        self.slider_cs = cs;

        self.slider_body = Some(graphics::SliderDrawable {
            slider_data,
            slider_grids,
            grid_cells,
            line_segments,
        });

        self.bounds = tataku::Bounds::new(
            min_pos,
            size,
        );
    }

    #[cfg(feature="graphics")]
    fn make_dots(&mut self) {
        self.hit_dots.clear();

        let mut slide_counter = 0;
        let mut moving_forwards = true;

        for t in self.curve.score_times.iter().copied() {
            // check for new slide
            let pos = (t - self.time) / (self.curve.length() / self.slides as f32);
            let current_moving_forwards = pos % 2.0 <= 1.0;
            if current_moving_forwards != moving_forwards {
                slide_counter += 1;
                moving_forwards = current_moving_forwards;
                // dont add dot if it conflicts with a repeat point
                continue
            }

            // dont add dot if it conflicts with the end circle
            if t == self.curve.end_time { continue }

            let dot = SliderDot::new(
                t,
                self.curve.position_at_time(t),
                slide_counter
            );

            self.hit_dots.push_back(dot);
        }

        self.dot_count = self.hit_dots.len();
    }

    pub fn update(&mut self, shell: &mut UpdateShell) {
        #[cfg(feature="graphics")] {
            if self.slider_body.is_none() || self.slider_cs != shell.coords.cs {
                self.make_body(shell.coords.cs);
            }
        }

        self.shake.update(shell.time);
        self.circle.update(shell);

        if shell.time < self.time { return; }

        // todo: check where can use the same sliding_ok
        // check sliding ok
        let slider_ball_pos = shell.coords.to_window(
            self.curve.position_at_time(shell.time)
        );

        let distance = slider_ball_pos.distance_squared(shell.cursor_pos);
        let radius = CIRCLE_RADIUS_BASE * shell.coords.cs;

        self.sliding_ok = shell.holding && distance <= (radius * OK_RADIUS_MULT).powi(2);

        // check if the start of the slider was missed.
        // if it was, perform a miss
        if self.initial_hit.is_none() && shell.time >= self.time + shell.hitwindow_miss {
            self.initial_hit = Some(OsuHitJudgments::Miss);
        }

        if shell.time < self.curve.end_time {
            // find out if a slide has been completed
            let pos = (shell.time - self.time) / (self.curve.length() / self.slides as f32);

            let current_moving_forwards = pos % 2.0 < 1.0;

            if current_moving_forwards != self.moving_forward {
                // direction changed
                self.moving_forward = current_moving_forwards;
                self.slides_complete += 1;

                // TODO: calc this properly? use to determine if ripple is on visual start or end
                let pos = slider_ball_pos;

                // check cursor
                if self.sliding_ok {
                    self.pending_combo.push((OsuHitJudgments::SliderEnd, pos));

                    let index = self.slides_complete.min(self.edge_sets.len() as u64 - 1) as usize;

                    let [normal_set, addition_set] = self.edge_sets[index];

                    let mut samples = self.hitsamples.clone();
                    samples.normal_set = normal_set;
                    samples.addition_set = addition_set;

                    let hitsound = self.edge_sounds[index.min(self.edge_sounds.len() - 1)];

                    let hitsounds = Hitsound::from_hitsamples(
                        hitsound,
                        samples,
                        true,
                        shell.timing_points.timing_point_at(shell.time, true),
                    );

                    shell.hitsounds.extend(hitsounds);

                    // self.add_ripple(beatmap_time, pos, false);
                } else {
                    // we broke combo
                    self.pending_combo.push((OsuHitJudgments::SliderEndMiss, pos));
                }
            }
        }

        #[cfg(feature="graphics")]
        while let Some(dot) = self.hit_dots.front() {
            if shell.time < dot.time { break; }

            let pos = shell.coords.to_window(dot.pos);

            let distance = pos.distance_squared(shell.cursor_pos);

            let sliding_ok = shell.holding && distance <= (radius * OK_TICK_RADIUS_MULT).powi(2);

            if sliding_ok {
                self.pending_combo.push((OsuHitJudgments::SliderDot, dot.pos));

                const SAMPLE_SETS:[&str; 4] = ["normal", "normal", "soft", "drum"];

                let sliderdot_hitsound = Hitsound::new_simple(format!("{}-slidertick", SAMPLE_SETS[self.hitsamples.addition_set as usize]));

                shell.hitsounds.push(sliderdot_hitsound);

                // self.add_ripple(beatmap_time, dot.pos, true);
            } else {
                self.pending_combo.push((OsuHitJudgments::SliderDotMiss, dot.pos));
                self.dots_missed += 1;
            }

            self.hit_dots.pop_front();
        }

        // Check final judgement
        if shell.time >= self.curve.end_time && self.pending_combo.len() == self.dot_count {
            let time_end_pos = shell.coords.to_window(self.time_end_pos);

            let distance = time_end_pos.distance_squared(shell.cursor_pos);

            let sliding_ok = shell.holding && distance <= (radius * OK_RADIUS_MULT).powi(2);

            let start_judgement = self.initial_hit.unwrap_or(OsuHitJudgments::Miss);

            let judge = if start_judgement == OsuHitJudgments::Miss {
                if self.dot_count == 0 {
                    if sliding_ok {
                        OsuHitJudgments::X100
                    } else {
                        OsuHitJudgments::Miss
                    }
                } else if self.dots_missed == self.dot_count {
                    OsuHitJudgments::Miss
                } else if self.dots_missed == 0 {
                    // self.add_end_ripple(time);
                    OsuHitJudgments::X100
                } else {
                    // self.add_end_ripple(time);
                    OsuHitJudgments::X50
                }
            } else if self.dots_missed == 0 && sliding_ok {
                // self.add_end_ripple(time);
                OsuHitJudgments::X300
            } else {
                // self.add_end_ripple(time);
                OsuHitJudgments::X100
            };

            self.pending_combo.push((judge, self.time_end_pos));
        }
    }

    #[cfg(feature="graphics")]
    pub fn draw(&mut self, shell: &mut DrawShell) {
        if self.time - shell.time > shell.fade_in_time
            || shell.time - self.curve.end_time > shell.fade_out_time
        {
            return
        }

        let base_transform = shell.transform * self.shake.transform(2.0);

        let beat_scale = f32::lerp(
            BEAT_SCALE,
            1.0,
            (shell.time - shell.last_beat) / shell.pulse_length
        ).clamp(1.0, BEAT_SCALE);

        // color
        let alpha = shell.alpha(self.time, self.curve.end_time);
        let color = shell.combo_color(self.color_index)
            .with_alpha(alpha);

        // slider body
        if let Some(mut slider) = self.slider_body.clone() {
            let mut color: Color = shell.slider_track_override.filter(
                |c| c != &Color::BLACK
                    && shell.settings.use_skin_slider_body_color
            ).unwrap_or_else(|| {
                const DARKER:f32 = 2.0/3.0;
                color * DARKER
            });
            color.a = ColorField::new_f32(shell.settings.slider_body_alpha);

            let border_color = BORDER_COLOR.with_alpha_f32(shell.settings.slider_border_alpha);

            slider.slider_data.body_color = color;
            slider.slider_data.border_color = border_color;

            let transform = graphics::Transform {
                scale: slider.slider_data.size,
                ..graphics::Transform::IDENTITY
            };

            let transform2 = graphics::Transform {
                origin: self.pos - self.bounds.pos,
                scale: Vector2::ONE * shell.coords.scale,
                pos: shell.coords.to_window(self.pos),
                ..graphics::Transform::IDENTITY
            };

            slider.slider_data.scale = shell.coords.scale;

            let transform = transform2.matrix() * transform.matrix();

            shell.list.push(slider
                .with_transform(base_transform * transform)
                .merge_draw_options(graphics::DrawOptions {
                    alpha: Some(alpha),
                    ..Default::default()
                })
            );
        }

        // draw hit dots
        for dot in self.hit_dots.iter() {
            if dot.slide_layer > self.slides_complete { break; };

            let pos = shell.coords.to_window(dot.pos);

            if dot.slide_layer == self.slides_complete {
                if let Some(image) = self.dot_image.clone() {
                    let transform = graphics::Transform {
                        origin: image.size() / 2.0,
                        scale: Vector2::ONE * beat_scale * 0.8,
                        pos,
                        ..graphics::Transform::IDENTITY
                    };

                    shell.list.push(image.with_transform(
                        base_transform * transform.matrix()
                    ));
                } else {
                    let transform = graphics::Transform {
                        scale: Vector2::ONE * SLIDER_DOT_RADIUS * beat_scale,
                        pos,
                        ..graphics::Transform::IDENTITY
                    };

                    shell.list.push(graphics::Circle::new(Color::YELLOW)
                        .border(Border::new(Color::WHITE, OSU_NOTE_BORDER_SIZE))
                        .with_transform(base_transform * transform.matrix())
                    );
                }
            }
        }

        // start and end circles
        let slides_remaining = self.slides - self.slides_complete;
        let end_repeat = slides_remaining > self.slides % 2 + 1;
        let start_repeat = slides_remaining > 2 - self.slides % 2;

        let pos = shell.coords.to_window(self.pos);
        let visual_end_pos = shell.coords.to_window(self.visual_end_pos);

        // end pos
        if let Some(mut end_circle) = self.end_circle_image.clone() {
            end_circle.color = color;

            let transform = graphics::Transform {
                origin: end_circle.size() / 2.0, // center
                scale: Vector2::ONE * shell.coords.cs,
                pos: visual_end_pos,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(end_circle.with_transform(
                base_transform * transform.matrix()
            ));
        } else if self.circle.circle.is_none() {
            let transform = graphics::Transform {
                scale: Vector2::ONE * CIRCLE_RADIUS_BASE * shell.coords.cs,
                pos: visual_end_pos,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(graphics::Circle::new(color)
                .border(Border::new(
                    if end_repeat { Color::YELLOW } else { Color::WHITE }.with_alpha(alpha),
                    shell.coords.border_width
                ))
                .with_transform(base_transform * transform.matrix())
            );
        }

        if end_repeat
        && let Some(mut reverse_arrow) = self.slider_reverse_image.clone() {
            reverse_arrow.color.a = alpha;

            let l = self.curve.curve_lines.last().unwrap();

            let transform = graphics::Transform {
                origin: reverse_arrow.size() / 2.0, // center
                scale: Vector2::ONE * shell.coords.cs * beat_scale,
                rotation: (l.p1 - l.p2).atan2_wrong(),
                pos: visual_end_pos,
            };

            shell.list.push(reverse_arrow.with_transform(
                base_transform * transform.matrix()
            ));
        }

        // start pos
        if shell.time < self.time {
            // draw the starting circle as a hitcircle
            let transform = base_transform * graphics::Transform {
                scale: Vector2::ONE * shell.coords.cs,
                pos,
                ..graphics::Transform::IDENTITY
            }.matrix();

            let mut shell = DrawShell {
                transform,
                alpha,
                list: shell.list,
                ..*shell
            };

            self.circle.draw(self.color_index, &mut shell);
        } else {
            // draw it as a slider end
            if let Some(end_circle) = &self.end_circle_image {
                let mut end_circle = end_circle.clone();
                end_circle.color = color;

                let transform = graphics::Transform {
                    origin: end_circle.size() / 2.0, // center
                    scale: Vector2::ONE * shell.coords.cs,
                    pos: visual_end_pos,
                    ..graphics::Transform::IDENTITY
                };

                shell.list.push(end_circle.with_transform(
                    base_transform * transform.matrix()
                ));
            } else if self.circle.circle.is_none() {
                let transform = graphics::Transform {
                    scale: Vector2::ONE * CIRCLE_RADIUS_BASE * shell.coords.cs,
                    pos: visual_end_pos,
                    ..graphics::Transform::IDENTITY
                };

                shell.list.push(graphics::Circle::new(color)
                    .border(Border::new(
                        if start_repeat { Color::YELLOW } else { Color::WHITE }.with_alpha(alpha),
                        shell.coords.border_width
                    ))
                    .with_transform(base_transform * transform.matrix())
                );
            }

            if start_repeat
            && let Some(mut reverse_arrow) = self.slider_reverse_image.clone() {
                reverse_arrow.color.a = alpha;

                let l = self.curve.curve_lines.first().unwrap();

                let transform = graphics::Transform {
                    origin: reverse_arrow.size() / 2.0, // center
                    scale: Vector2::ONE * shell.coords.cs * beat_scale,
                    rotation: (l.p1 - l.p2).atan2_wrong(),
                    pos: visual_end_pos,
                };

                shell.list.push(reverse_arrow.with_transform(
                    base_transform * transform.matrix()
                ));
            }
        }

        // slider ball
        if shell.time >= self.time && shell.time < self.curve.end_time {
            let slider_ball_pos = self.curve.position_at_time(shell.time);

            let direction = self.curve.position_at_time(shell.time + 0.1) - slider_ball_pos;
            let slider_ball_pos = shell.coords.to_window(slider_ball_pos);

            let rotation = PI * 2.0 - direction.atan2();

            // under
            if let Some(mut ball) = self.sliderball_under_image.clone() {
                ball.color = color;

                let transform = graphics::Transform {
                    origin: ball.size() / 2.0, // center
                    scale: Vector2::ONE * shell.coords.cs,
                    pos: slider_ball_pos,
                    ..graphics::Transform::IDENTITY
                };

                shell.list.push(ball.with_transform(
                    base_transform * transform.matrix()
                ));
            }

            // inner
            if let Some(mut ball) = self.sliderball_image.clone() {
                ball.color = color;

                let transform = graphics::Transform {
                    origin: ball.size() / 2.0,
                    scale: Vector2::ONE * shell.coords.cs,
                    rotation,
                    pos: slider_ball_pos,
                };

                shell.list.push(ball.with_transform(
                    base_transform * transform.matrix())
                );
            } else {
                let transform = graphics::Transform {
                    scale: Vector2::ONE * CIRCLE_RADIUS_BASE * shell.coords.cs,
                    pos: slider_ball_pos,
                    ..graphics::Transform::IDENTITY
                };

                shell.list.push(graphics::Circle::new(color)
                    .border(Border::new(Color::WHITE.with_alpha(alpha), 2.0))
                    .with_transform(base_transform * transform.matrix())
                );
            }

            // radius thingy
            if let Some(mut circle) = self.follow_circle_image.clone() {
                circle.color = color;

                let transform = graphics::Transform {
                    origin: circle.size() / 2.0, // center
                    scale: Vector2::ONE * shell.coords.cs,
                    rotation,
                    pos: slider_ball_pos,
                };

                shell.list.push(circle.with_transform(
                    base_transform * transform.matrix()
                ));
            } else {
                let transform = graphics::Transform {
                    scale: Vector2::ONE * CIRCLE_RADIUS_BASE * shell.coords.cs * OK_TICK_RADIUS_MULT,
                    pos: slider_ball_pos,
                    ..graphics::Transform::IDENTITY
                };

                shell.list.push(graphics::Circle::new(Color::TRANSPARENT)
                    .border(Border::new(
                        if self.sliding_ok {Color::LIME} else {Color::RED}.with_alpha(alpha),
                        2.0
                    ))
                    .with_transform(base_transform * transform.matrix())
                );
            }
        }

        // approach circle
        if shell.time < self.time {
            let transform = base_transform * engine::graphics::Transform {
                pos,
                scale: Vector2::ONE * shell.coords.cs,
                ..Default::default()
            }.matrix();

            let mut shell = DrawShell {
                transform,
                alpha,
                list: shell.list,
                ..*shell
            };

            self.approach_circle.draw(self.time, self.color_index, &mut shell);
        }
    }

    pub fn reset(&mut self) {
        self.pending_combo.clear();
        self.initial_hit = None;
        self.slides_complete = 0;
        self.moving_forward = true;

        self.dots_missed = 0;
        self.dot_count = 0;

        self.make_dots();
        self.slider_body = None;
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider,
    ) {
        use graphics::SkinUsage;

        self.circle.reload_skin(source, skin_manager);
        self.end_circle_image = skin_manager.get_texture(
            Path::new("sliderendcircle"),
            source,
            SkinUsage::Gamemode,
            false
        );
        self.slider_reverse_image = skin_manager.get_texture(
            Path::new("reversearrow"),
            source,
            SkinUsage::Gamemode,
            false
        );
        self.follow_circle_image = skin_manager.get_texture(
            Path::new("sliderfollowcircle"),
            source,
            SkinUsage::Gamemode,
            false
        );

        self.approach_circle.reload_texture(source, skin_manager);

        self.dot_image = skin_manager.get_texture(
            Path::new("sliderscorepoint"),
            source,
            graphics::SkinUsage::Gamemode,
            false
        );

        // slider ball
        self.sliderball_under_image = skin_manager.get_texture(
            Path::new("sliderb-nd"),
            source,
            SkinUsage::Gamemode,
            false
        );

        let mut i = 0;
        let mut images = Vec::new();
        loop {
            let name = format!("sliderb{i}");
            let Some(image) = skin_manager.get_texture(Path::new(&name), source, SkinUsage::Gamemode, false) else { break };
            images.push(image);
            i += 1;
        }

        if !images.is_empty() {
            let base_scale = images[0].base_scale;

            let images = images.into_iter().map(|i|i.tex).collect::<Vec<_>>();

            // stolen from peppy, i'll figure it out later lol
            let frametime = 1000.0 / 60.0;
            let velocity = self.speed;
            let frametime = ((150.0 / velocity) * frametime).max(frametime);

            let animation = graphics::Animation::new(
                images,
                frametime,
                base_scale
            );

            self.sliderball_image = Some(animation);
        } else {
            self.sliderball_image = None;
        }

        self.slider_body = None;
    }
}
#[cfg(feature="graphics")]
/// struct for drawing hit slider points
#[derive(Clone)]
struct SliderDot {
    time: f32,
    pos: Vector2,

    /// which slide "layer" is this on?
    slide_layer: u64,
}

#[cfg(feature="graphics")]
impl SliderDot {
    pub fn new(time: f32, pos: Vector2, slide_layer: u64) -> Self {
        Self {
            time,
            pos,
            slide_layer,
        }
    }
}
