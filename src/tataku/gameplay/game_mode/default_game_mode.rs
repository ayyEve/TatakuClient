use crate::prelude::*;


// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode;

#[async_trait]
impl GameMode for NoMode {
    async fn new(_:&Beatmap, _:bool) -> Result<Self, TatakuError> where Self: Sized {Ok(Self {})}

    async fn handle_replay_frame<'a>(&mut self, _: ReplayFrame, _: &mut GameplayStateForUpdate<'a>) {}
    async fn update<'a>(&mut self, _: &mut GameplayStateForUpdate<'a>) { }
    async fn draw<'a>(&mut self, _:GameplayStateForDraw<'a>, _: &mut RenderableCollection) {}
    fn skip_intro(&mut self, _: f32) -> Option<f32> { None }
    async fn reset(&mut self, _:&Beatmap) {}
    async fn window_size_changed(&mut self, _: Arc<WindowSize>) {}
    async fn fit_to_area(&mut self, _:Bounds) {}
    async fn force_update_settings(&mut self, _: &Settings) {}
    
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, _beatmap_folder: &String, _skin_manager: &mut SkinManager) -> TextureSource { TextureSource::Raw }
    async fn apply_mods(&mut self, _: Arc<ModManager>) {}

    
    async fn beat_happened(&mut self, _pulse_length: f32) {}
    async fn kiai_changed(&mut self, _is_kiai: bool) {}
}

#[cfg(feature="graphics")]
#[async_trait]
impl GameModeInput for NoMode {
    async fn key_down(&mut self, _:Key) -> Option<ReplayAction> { None }
    async fn key_up(&mut self, _:Key) -> Option<ReplayAction> { None }
}
#[cfg(not(feature="graphics"))]
impl GameModeInput for NoMode {}


impl GameModeProperties for NoMode {
    fn playmode(&self) -> Cow<'static, str> { Cow::Borrowed("none") }
    fn end_time(&self) -> f32 {0.0}
    
    // fn combo_bounds(&self) -> Rectangle {SimpleRectangle::new(Vector2::ZERO, Vector2::ZERO)}
    fn timing_bar_things(&self) -> Vec<(f32,Color)> { Vec::new() }
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> { Vec::new() }
    // fn score_hit_string(_hit:&ScoreHit) -> String where Self: Sized { String::new() }
}
