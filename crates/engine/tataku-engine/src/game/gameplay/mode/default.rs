use crate::*;
use common::replays::{ ReplayAction, ReplayFrame };
use engine::gameplay::{
    properties::GamemodeProperties,
    gameplay_manager::*,
};

// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode;
impl gameplay::GameMode for NoMode {
    fn new(
        _: &beatmaps::Beatmap, 
        _: bool, 
        _: &Settings
    ) -> Result<Self, tataku::Error> where Self: Sized {
        Ok(Self {})
    }

    fn handle_replay_frame(&mut self, _: ReplayFrame, _: &mut GameplayUpdateShell) {}
    fn update(&mut self, _: &mut GameplayUpdateShell) { }
    #[cfg(feature="graphics")]
    fn draw(&mut self, _: GameplayDrawShell, _: &mut graphics::RenderableCollection) {}
    #[cfg(feature="gameplay")] 
    fn skip_intro(&mut self, _: f32) -> Option<f32> { None }

    fn reset(&mut self, _: &beatmaps::Beatmap) {}
    fn force_update_settings(&mut self, _: &Settings) {}
    fn handle_gameplay_event(&mut self, _: gameplay::GameplayEvent) {}
    
    #[cfg(feature="graphics")]
    fn reload_skin(&mut self, _: &str, _: &mut dyn graphics::SkinProvider) -> graphics::TextureSource { 
        graphics::TextureSource::Raw 
    }

    #[cfg(feature="graphics")] 
    fn get_playfield(&self) -> gameplay::PlayfieldNonsense { gameplay::PlayfieldNonsense::default() }
    fn properties(&self, _: &gameplay::TimingPointHelper) -> GamemodeProperties { GamemodeProperties::default() }

    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, _input: input::InputEvent) -> Option<ReplayAction> { None }
}
