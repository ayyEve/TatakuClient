use crate::*;

#[cfg(feature="gameplay")] 
use common::replays::ReplayAction;

#[cfg(feature="gameplay")] 
use engine::gameplay::gameplay_manager::GameplayDrawShell;

use engine::gameplay::{
    helpers::*,
    gameplay_manager::{
        GameplayUpdateShell,
        GameplayCustomShell,
    },
};


pub trait Gamemode: Send + Sync {
    fn new(
        beatmap: &beatmaps::Beatmap, 
        diff_calc_only: bool,
        settings: &Settings,
    ) -> Result<Self, tataku::Error> where Self:Sized;

    fn handle_replay_frame(
        &mut self, 
        frame: common::replays::ReplayFrame, 
        state: &mut GameplayUpdateShell
    );

    fn handle_gameplay_event(&mut self, shell: &mut GameplayCustomShell<'_, gameplay::GameplayEvent>);

    fn update(&mut self, shell: &mut GameplayUpdateShell);

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        shell: GameplayDrawShell, 
        list: &mut graphics::RenderableCollection,
    );

    #[cfg(feature="gameplay")] 
    fn skip_intro(&mut self, time: f32) -> Option<f32>;
    fn reset(&mut self, shell: &GameplayCustomShell<'_, &beatmaps::Beatmap>);
    
    fn force_update_settings(&mut self, settings: &Settings);

    #[cfg(feature="graphics")]
    fn reload_skin(
        &mut self, 
        beatmap_path: &str, 
        skin_manager: &mut dyn graphics::SkinProvider
    ) -> graphics::TextureSource;

    fn properties(&self, timing_points: &TimingPointHelper) -> gameplay::mode::GamemodeProperties;
    fn time_jump(&mut self, _new_time: f32, _state: &mut GameplayUpdateShell) {}

    #[cfg(feature="graphics")] fn get_playfield(&self) -> PlayfieldNonsense;

    /// setup any gamemode specific ui elements for this gamemode
    /// ie combo and leaderboard, since the pos is different per-mode
    #[cfg(feature="graphics")]
    fn build_widgets(&self, _loader: &mut gameplay::widgets::UiElementLoader) {}

    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, input: input::InputEvent) -> Option<ReplayAction>;

    fn all_notes(&self) -> Vec<&dyn gameplay::HitObject>;
}

