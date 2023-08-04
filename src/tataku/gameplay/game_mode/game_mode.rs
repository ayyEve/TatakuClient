use crate::prelude::*;

pub type PerformanceCalc = Box<fn(f32, f32) -> f32>;


#[async_trait]
pub trait GameMode: GameModeInput + GameModeProperties + Send + Sync {
    async fn new(beatmap:&Beatmap, diff_calc_only: bool) -> Result<Self, TatakuError> where Self:Sized;

    async fn handle_replay_frame(&mut self, frame:ReplayAction, time:f32, manager:&mut IngameManager);

    async fn update(&mut self, manager:&mut IngameManager, time: f32) -> Vec<ReplayAction>;
    async fn draw(&mut self, manager:&mut IngameManager, list: &mut RenderableCollection);

    fn skip_intro(&mut self, manager: &mut IngameManager);
    fn pause(&mut self, _manager:&mut IngameManager) {}
    fn unpause(&mut self, _manager:&mut IngameManager) {}
    async fn reset(&mut self, beatmap:&Beatmap);

    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>);
    async fn fit_to_area(&mut self, _pos: Vector2, _size: Vector2);

    
    async fn force_update_settings(&mut self, settings: &Settings);
    async fn reload_skin(&mut self);

    async fn time_jump(&mut self, _new_time: f32) {}
    async fn apply_mods(&mut self, mods: Arc<ModManager>);
    // fn apply_auto(&mut self, settings: &BackgroundGameSettings);

    /// happens right when a beat occurs (or a bit after if theres lag/stutter)
    async fn beat_happened(&mut self, pulse_length: f32);
    /// happens right when kiai changes
    async fn kiai_changed(&mut self, is_kiai: bool);

}
impl Default for Box<dyn GameMode> {
    fn default() -> Self {
        Box::new(NoMode::default())
    }
}
