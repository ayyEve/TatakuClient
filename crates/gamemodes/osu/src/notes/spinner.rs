/*
 * Authored by ayyEve, RPM calculation by Nebula
*/

use std::f32::consts::PI;
use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Vector2,
};
use engine::{
    beatmaps::{
        osu::*,
    },
    gameplay::{
        judgments::HitJudgment,
    },
    ui,
};

use interface::parley;

#[cfg(feature="graphics")]
use engine::graphics;

const SPINNER_RADIUS:f32 = 200.0;

#[derive(Clone, Default)]
pub struct Spinner {
    pub time: f32,
    pub end_time: f32,

    last_update: f32,

    /// display rotation of the spinner (for display only)
    display_rotation: f32,
    /// current rotation of the spinner (for calc only)
    rotation: f32,
    /// Current and previous rotation windows.
    rotation_windows: [f32; 2],
    /// time start of current window
    window_start: f32,
    /// how fast the spinner is spinning
    rotation_velocity: f32,

    /// what was the last rotation value?
    last_mouse_angle: f32,
    /// how many rotations is needed to pass this spinner
    pub rotations_required: u16,
    /// how many rotations have been completed?
    pub rotations_completed: u16,
    pub judged: bool,

    /// main spinny
    #[cfg(feature="graphics")] spinner_circle: Option<graphics::Image>,
    /// bg, no spin
    #[cfg(feature="graphics")] spinner_background: Option<graphics::Image>,
    /// also bg, no spin
    #[cfg(feature="graphics")] spinner_bottom: Option<graphics::Image>,
    /// gets smaller towards end of spinner, from 100% to 0%
    #[cfg(feature="graphics")] spinner_approach: Option<graphics::Image>,


    #[cfg(feature = "graphics")] rpm_layout: Option<Arc<parley::Layout<Color>>>,
    #[cfg(feature = "graphics")] rpm_text_size: Vector2,

    pub points_queue: Vec<HitJudgment>
}

impl Spinner {
    pub fn new(
        def: &SpinnerDef,
        rotations_required: u16
    ) -> Self {
        let time = def.time;
        let end_time = def.end_time;

        Self {
            time,
            end_time,

            rotations_required,

            ..Self::default()
        }
    }

    fn text_layout(
        text: &str,
        scale: f32,
        context: &mut ui::widget::TextLayoutContexts,
    ) -> (Arc<parley::Layout<Color>>, Vector2) {
        let style = ui::style::TextStyle {
            font_size: 30.0 * scale,
            ..Default::default()
        };

        let mut layout = context.simple_text(
            text,
            &style,
        );
        layout.break_all_lines(None);

        let size = Vector2::new(
            layout.width(),
            layout.height(),
        );

        (Arc::new(layout), size)
    }

    pub fn update(&mut self, shell: &mut UpdateShell) {
        const WINDOW_PERIOD_MILLIS: f32 = 1000.0;

        if self.judged { return; }

        let cursor_pos = shell.coords.to_osu(shell.cursor_pos);

        let center = FIELD_SIZE / 2.0;

        let mut diff = 0.0;
        let pos_diff = cursor_pos - center;
        let mouse_angle = pos_diff.y.atan2(pos_diff.x);

        if shell.time >= self.time && shell.time <= self.end_time {
            // if the mouse is being held, use the mouse angle.
            // otherwise, it should be 0 since the user's spins do not count
            if shell.holding {
                diff = mouse_angle - self.last_mouse_angle;
            }
            // fix diff (this is stupid)
            if diff > PI { diff -= 2.0 * PI }
            else if diff < -PI { diff += 2.0 * PI }

            // Sliding window algorithm
            // Used to approximate rotation delta over
            // longer time, without needing to store
            // a queue of frame data.

            let time_delta = shell.time - self.window_start;
            let ratio = time_delta / WINDOW_PERIOD_MILLIS;

            if ratio >= 1.0 {
                if ratio >= 2.0 {
                    // Blank windows
                    self.rotation_windows = [0.0; 2];
                }
                else {
                    // Left shift
                    self.rotation_windows[0] = self.rotation_windows[1];
                    self.rotation_windows[1] = 0.0;
                }
            }

            self.rotation_windows[1] += diff;

            let current_window_proportion = ratio.fract();
            let previous_window_proportion = 1.0 - current_window_proportion;

            self.window_start = shell.time - WINDOW_PERIOD_MILLIS * current_window_proportion;

            let amount = self.rotation_windows[0] * previous_window_proportion + self.rotation_windows[1];

            self.rotation_velocity = amount / WINDOW_PERIOD_MILLIS;

            // update display rotation
            self.display_rotation += self.rotation_velocity * (shell.time - self.last_update);
            if self.display_rotation >= PI * 2.0 || self.display_rotation <= -PI * 2.0 {
                self.rotations_completed += 1;

                if self.rotations_completed >= self.rotations_required && (self.rotations_completed - self.rotations_required).is_multiple_of(3) {
                    self.points_queue.push(OsuHitJudgments::SpinnerPoint);
                }
            }
            self.display_rotation %= PI * 2.0;

            // update calc rotation
            self.rotation += diff;
            // if self.rotation >= PI * 2.0 || self.rotation <= -PI * 2.0 { self.rotations_completed += 1; }
            self.rotation %= PI * 2.0;

            let rpm = (self.rotation_velocity / (2.0 * PI)) * 1000.0 * 60.0;
            let (layout, size) = Self::text_layout(
                &format!("{:.0}rpm ({}/{})", rpm.abs(), self.rotations_completed, self.rotations_required),
                shell.coords.scale,
                shell.font_contexts,
            );

            self.rpm_layout = Some(layout);
            self.rpm_text_size = size;
        }

        self.last_mouse_angle = mouse_angle;
        self.last_update = shell.time;
    }

    #[cfg(feature="graphics")]
    pub fn draw(&self, shell: &mut DrawShell) {
        if shell.time < self.time || shell.time > self.end_time { return }

        let center = shell.coords.playfield.pos + shell.coords.playfield.size / 2.0;

        let scale = Vector2::ONE * shell.coords.cs;

        let border = Border::new(
            Color::BLACK,
            OSU_NOTE_BORDER_SIZE
        );

        // bg circle
        if let Some(i) = self.spinner_background.clone() {
            let transform = graphics::Transform {
                origin: i.size() / 2.0, // center
                scale,
                pos: center,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(i.with_transform(
                shell.transform * transform.matrix()
            ));
        }

        // bottom circle
        if let Some(i) = self.spinner_bottom.clone() {
            let transform = graphics::Transform {
                origin: i.size() / 2.0, // center
                scale,
                pos: center,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(i.with_transform(
                shell.transform * transform.matrix()
            ));
        } else if !(self.spinner_approach.is_some() || self.spinner_circle.is_some()) {
            let transform = graphics::Transform {
                scale: Vector2::ONE * SPINNER_RADIUS,
                pos: center,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(graphics::Circle::new(Color::YELLOW)
                .border(border)
                .with_transform(shell.transform * transform.matrix())
            );
        }


        // draw another circle on top which increases in radius as the counter gets closer to the reqired
        if let Some(i) = self.spinner_approach.clone() {
            let scale = scale *
                f32::lerp(
                    1.0,
                    0.0,
                    (shell.time - self.time) / (self.end_time - self.time)
                );

            let transform = graphics::Transform {
                origin: i.size() / 2.0, // center
                scale,
                pos: center,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(i.with_transform(
                shell.transform * transform.matrix()
            ));
        } else {
            let transform = graphics::Transform {
                scale: Vector2::ONE * SPINNER_RADIUS * (self.rotations_completed as f32 / self.rotations_required as f32).min(1.0),
                pos: center,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(graphics::Circle::new(Color::WHITE)
                .border(border)
                .with_transform(shell.transform * transform.matrix())
            );
        }


        // draw line to show rotation
        if let Some(i) = self.spinner_circle.clone() {
            let transform = graphics::Transform {
                origin: i.size() / 2.0, // center
                scale,
                rotation: self.display_rotation,
                pos: center,
            };

            shell.list.push(i.with_transform(
                shell.transform * transform.matrix()
            ));
        } else {
            let vector = Vector2::new(
                    self.display_rotation.cos(),
                    self.display_rotation.sin()
                )
                * SPINNER_RADIUS;

            let transform = graphics::Transform {
                pos: center,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(graphics::Line::new(
                vector,
                5.0,
                Color::GREEN
            ).with_transform(shell.transform * transform.matrix()));
        }

        // draw counter
        if let Some(layout) = self.rpm_layout.clone() {
            let transform = graphics::Transform {
                origin: self.rpm_text_size / 2.0,
                pos: center + Vector2::with_y(shell.coords.playfield.size.y / 4.0),
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(graphics::Text::new(layout)
                .with_transform(shell.transform * transform.matrix())
            );
        }
    }

    pub fn reset(&mut self) {
        self.rotation = 0.0;
        self.last_update = 0.0;
        self.last_mouse_angle = 0.0;
        self.display_rotation = 0.0;
        self.rotation_velocity = 0.0;
        self.rotations_completed = 0;
        self.rotation_windows = [0.0; 2];
        self.judged = false;

        self.rpm_layout = None;
        self.points_queue.clear();
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        self.spinner_circle = skin_manager.get_texture(
            Path::new("spinner-circle"),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        );

        self.spinner_background = skin_manager.get_texture(
            Path::new("spinner-background"),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        );

        self.spinner_bottom = skin_manager.get_texture(
            Path::new("spinner-bottom"),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        );

        self.spinner_approach = skin_manager.get_texture(
            Path::new("spinner-approachcircle"),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        );

    }
}
