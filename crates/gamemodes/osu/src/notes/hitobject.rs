use crate::prelude::*;

use tataku::{
    Color,
    ColorField,
    Interpolation,
};

use engine::gameplay::{ Hitsound, TimingPointProgress };

#[derive(From)]
pub enum HitObject {
    Circle(Note),
    Slider(Slider),
    Spinner(Spinner),
}

impl HitObject {
    pub fn time(&self) -> f32 {
        match self {
            HitObject::Circle(note) => note.time,
            HitObject::Slider(slider) => slider.time,
            HitObject::Spinner(spinner) => spinner.time,
        }
    }

    pub fn update(&mut self, shell: &mut UpdateShell) {
        match self {
            HitObject::Circle(note) => note.update(shell),
            HitObject::Slider(slider) => slider.update(shell),
            HitObject::Spinner(spinner) => spinner.update(shell),
        }
    }

    pub fn reset(&mut self) {
        match self {
            HitObject::Circle(note) => note.reset(),
            HitObject::Slider(slider) => slider.reset(),
            HitObject::Spinner(spinner) => spinner.reset(),
        }
    }

    pub fn reload_skin(
        &mut self,
        source: &tataku_graphics::TextureSource,
        skin_manager: &mut dyn tataku_graphics::SkinProvider,
    ) {
        match self {
            HitObject::Circle(note) => note.reload_skin(source, skin_manager),
            HitObject::Slider(slider) => slider.reload_skin(source, skin_manager),
            HitObject::Spinner(spinner) => spinner.reload_skin(source, skin_manager),
        }
    }
}

pub struct UpdateShell<'a> {
    pub time: f32,
    pub hitwindow_miss: f32,
    pub cursor_pos: tataku::Vector2,
    pub holding: bool,

    pub settings: &'a Settings,
    pub coords: &'a OsuCoords,
    pub timing_points: &'a TimingPointProgress,

    pub hitsounds: &'a mut Vec<Hitsound>,

    pub font_contexts: &'a mut engine::ui::widget::TextLayoutContexts
}

pub struct DrawShell<'a> {
    pub time: f32,
    pub fade_in_time: f32,
    pub fade_out_time: f32,
    pub hitwindow_miss: f32,

    pub alpha: ColorField,

    pub combo_colors: &'a [Color],

    pub last_beat: f32,
    pub pulse_length: f32,

    pub list: &'a mut engine::graphics::RenderableCollection,
    pub transform: tataku::Matrix,
    pub settings: &'a Settings,
    pub coords: &'a OsuCoords,
    pub slider_track_override: Option<Color>,
}

impl DrawShell<'_> {
    pub fn combo_color(&self, color_index: usize) -> Color {
        self.combo_colors[color_index % self.combo_colors.len()]
    }

    pub fn alpha(&self, note_time: f32, hit_time: f32) -> ColorField {
        let alpha = self.alpha.to_f32();

        let alpha = alpha
            * self.fade_in(note_time)
            * self.fade_out(hit_time);

        ColorField::new_f32(alpha)
    }

    pub fn fade_in(&self, note_time: f32) -> f32 {
        let t = (note_time - self.time) / self.fade_in_time;
        let t = t.clamp(1.0 / 3.0, 1.0);

        f32::lerp(
            1.0,
            0.0,
            (3.0 * t - 1.0) / 2.0,
        )
    }

    pub fn fade_out(&self, hit_time: f32) -> f32 {
        let t = (self.time - hit_time) / self.fade_out_time;
        let t = t.clamp(0.0, 1.0);

        f32::lerp(
            1.0,
            0.0,
            t,
        )
    }
}
