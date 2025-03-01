use crate::prelude::*;

#[async_trait]
pub trait GameMode: Send + Sync {
    async fn new(
        beatmap: &Beatmap, 
        diff_calc_only: bool,
        settings: &Settings,
    ) -> Result<Self, TatakuError> where Self:Sized;

    async fn handle_replay_frame<'a>(
        &mut self, 
        frame: ReplayFrame, 
        state: &mut GameplayUpdateShell<'a>
    );

    async fn update<'a>(
        &mut self, 
        state: &mut GameplayUpdateShell<'a>
    );

    async fn draw<'a>(
        &mut self, 
        state: GameplayDrawShell<'a>, 
        list: &mut RenderableCollection,
    );

    fn skip_intro(&mut self, time: f32) -> Option<f32>;
    fn pause(&mut self) {}
    fn unpause(&mut self) {}
    async fn reset(&mut self, beatmap: &Beatmap);

    fn set_bounds(&mut self, bounds: Bounds, full_window: bool);
    
    async fn force_update_settings(&mut self, settings: &Settings);
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, beatmap_path: &str, skin_manager: &mut dyn SkinProvider) -> TextureSource;

    async fn time_jump<'a>(&mut self, _new_time: f32, _state: &mut GameplayUpdateShell<'a>) {}
    async fn apply_mods(&mut self, mods: Arc<ModManager>);
    // fn apply_auto(&mut self, settings: &BackgroundGameSettings);

    /// happens right when a beat occurs (or a bit after if theres lag/stutter)
    async fn beat_happened(&mut self, pulse_length: f32);
    /// happens right when kiai changes
    async fn kiai_changed(&mut self, is_kiai: bool);

    fn properties(&self) -> GameModeProperties;

    fn get_playfield(&self) -> PlayfieldNonsense;

    /// setup any gamemode specific ui elements for this gamemode
    /// ie combo and leaderboard, since the pos is different per-mode
    async fn build_widgets(
        &self, 
        _loader: &mut dyn UiElementLoader,
    ) {}


    async fn handle_input(&mut self, input: InputEvent) -> Option<ReplayAction>;
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
