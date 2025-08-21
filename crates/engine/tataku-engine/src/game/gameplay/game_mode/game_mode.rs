use crate::prelude::*;
#[cfg(feature = "graphics")] use tataku_graphics::prelude::*;

pub trait GameMode: Send + Sync {
    fn new(
        beatmap: &Beatmap, 
        diff_calc_only: bool,
        settings: &Settings,
    ) -> Result<Self, TatakuError> where Self:Sized;

    fn handle_replay_frame(
        &mut self, 
        frame: ReplayFrame, 
        state: &mut GameplayUpdateShell
    );

    fn handle_gameplay_event(&mut self, event: GameplayEvent);

    fn update(
        &mut self, 
        state: &mut GameplayUpdateShell
    );

    #[cfg(feature="graphics")]
    fn draw(
        &mut self, 
        state: GameplayDrawShell, 
        list: &mut RenderableCollection,
    );

    #[cfg(feature="gameplay")] 
    fn skip_intro(&mut self, time: f32) -> Option<f32>;
    fn reset(&mut self, beatmap: &Beatmap);
    
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
        skin_manager: &mut dyn SkinProvider
    ) -> TextureSource;

    fn properties(&self, timing_points: &TimingPointHelper) -> GameModeProperties;
    fn time_jump(&mut self, _new_time: f32, _state: &mut GameplayUpdateShell) {}

    #[cfg(feature="graphics")] fn get_playfield(&self) -> PlayfieldNonsense;

    /// setup any gamemode specific ui elements for this gamemode
    /// ie combo and leaderboard, since the pos is different per-mode
    #[cfg(feature="graphics")]
    fn build_widgets(&self, _loader: &mut dyn UiElementLoader) {}

    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, input: InputEvent) -> Option<ReplayAction>;
}


#[derive(ChainableInitializer)]
#[derive(Copy, Clone, Debug, Default)]
pub struct PlayfieldNonsense {
    pub bounds: Bounds,
    #[chain] pub scale: f32,
    #[chain] pub circle_size: Vector2,
    #[chain] pub flip_vertical: bool,
    #[chain] pub is_fullscreen: bool,
}
impl PlayfieldNonsense {
    pub fn new(
        bounds: Bounds, 
        scale: f32, 
        circle_size: Vector2,
        flip_vertical: bool,
    ) -> Self {
        Self {
            bounds,
            scale,
            circle_size,
            flip_vertical,
            is_fullscreen: false,
        }
    }

    pub fn new_simple(bounds: Bounds) -> Self {
        Self {
            bounds,
            scale: 1.0,
            ..Default::default()
        }
    }
}
