use crate::prelude::*;

#[cfg(feature="graphics")]
use engine::graphics;

#[derive(Clone, From)]
pub enum HitObject {
    Note(super::Note),
    Drumroll(super::Drumroll),
    Spinner(super::Spinner),
}

impl HitObject {
    pub fn time(&self) -> f32 {
        match self {
            HitObject::Note(note) => note.time,
            HitObject::Drumroll(drumroll) => drumroll.time,
            HitObject::Spinner(spinner) => spinner.time,
        }
    }

    pub fn note_type(&self) -> engine::beatmaps::NoteType {
        use engine::beatmaps::NoteType;

        match self {
            HitObject::Note(_) => NoteType::Note,
            HitObject::Drumroll(_) => NoteType::Slider,
            HitObject::Spinner(_) => NoteType::Spinner,
        }
    }

    pub fn get_speed(&self) -> f32 {
        match self {
            HitObject::Note(note) => note.speed,
            HitObject::Drumroll(drumroll) => drumroll.speed,
            HitObject::Spinner(spinner) => spinner.speed,
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        match self {
            HitObject::Note(note) => note.speed = speed,
            HitObject::Drumroll(drumroll) => drumroll.speed = speed,
            HitObject::Spinner(spinner) => spinner.speed = speed,
        }
    }

    pub fn draw(&self, shell: &mut DrawShell) {
        match self {
            HitObject::Note(note) => note.draw(shell),
            HitObject::Drumroll(drumroll) => drumroll.draw(shell),
            HitObject::Spinner(spinner) => spinner.draw(shell),
        }
    }

    pub fn reset(&mut self) {
        match self {
            HitObject::Note(note) => note.reset(),
            HitObject::Drumroll(drumroll) => drumroll.reset(),
            HitObject::Spinner(spinner) => spinner.reset(),
        }
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        match self {
            HitObject::Note(note) => note.reload_skin(source, skin_manager),
            HitObject::Drumroll(drumroll) => drumroll.reload_skin(source, skin_manager),
            HitObject::Spinner(spinner) => spinner.reload_skin(source, skin_manager),
        }
    }
}

pub struct DrawShell<'a> {
    pub time: f32,
    pub list: &'a mut engine::graphics::RenderableCollection,
    pub settings: &'a Settings,
    pub playfield: &'a Playfield,
}
