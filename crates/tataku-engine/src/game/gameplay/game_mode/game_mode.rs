use crate::prelude::*;

#[async_trait]
pub trait GameMode: GameModeInput + GameModeProperties + Send + Sync {
    async fn new(
        beatmap: &Beatmap, 
        diff_calc_only: bool,
        settings: &Settings,
    ) -> Result<Self, TatakuError> where Self:Sized;

    async fn handle_replay_frame<'a>(
        &mut self, 
        frame: ReplayFrame, 
        state: &mut GameplayStateForUpdate<'a>
    );

    async fn update<'a>(
        &mut self, 
        state: &mut GameplayStateForUpdate<'a>
    );

    async fn draw<'a>(
        &mut self, 
        state: GameplayStateForDraw<'a>, 
        list: &mut RenderableCollection,
    );

    fn skip_intro(&mut self, time: f32) -> Option<f32>;
    fn pause(&mut self) {}
    fn unpause(&mut self) {}
    async fn reset(&mut self, beatmap: &Beatmap);

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>);
    async fn fit_to_area(&mut self, bounds: Bounds);

    
    async fn force_update_settings(&mut self, settings: &Settings);
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, beatmap_path: &str, skin_manager: &mut dyn SkinProvider) -> TextureSource;

    async fn time_jump(&mut self, _new_time: f32) {}
    async fn apply_mods(&mut self, mods: Arc<ModManager>);
    // fn apply_auto(&mut self, settings: &BackgroundGameSettings);

    /// happens right when a beat occurs (or a bit after if theres lag/stutter)
    async fn beat_happened(&mut self, pulse_length: f32);
    /// happens right when kiai changes
    async fn kiai_changed(&mut self, is_kiai: bool);

}
