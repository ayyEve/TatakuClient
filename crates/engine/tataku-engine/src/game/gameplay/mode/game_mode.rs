use crate::*;
use common::replays::ReplayAction;

use engine::gameplay::{
    helpers::*,
    gameplay_manager::{
        GameplayUpdateShell,
        GameplayDrawShell,
    },
};


pub trait GameMode: Send + Sync {
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

    fn handle_gameplay_event(&mut self, event: gameplay::GameplayEvent);

    fn update(
        &mut self, 
        state: &mut GameplayUpdateShell
    );

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        state: GameplayDrawShell, 
        list: &mut graphics::RenderableCollection,
    );

    #[cfg(feature="gameplay")] 
    fn skip_intro(&mut self, time: f32) -> Option<f32>;
    fn reset(&mut self, beatmap: &beatmaps::Beatmap);
    
    // fn pause(&mut self) {}
    // fn unpause(&mut self) {}
    // #[cfg(feature="graphics")]
    // fn set_bounds(&mut self, bounds: Bounds, full_window: bool);
    // fn apply_mods(&mut self, mods: Arc<ModManager>);

    // /// happens right when a beat occurs (or a bit after if theres lag/stutter)
    // fn beat_happened(&mut self, pulse_length: f32);
    // /// happens right when kiai changes
    // fn kiai_changed(&mut self, is_kiai: bool);
    
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
    fn build_widgets(&self, _loader: &mut dyn gameplay::widgets::UiElementLoader) {}

    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, input: input::InputEvent) -> Option<ReplayAction>;
}
