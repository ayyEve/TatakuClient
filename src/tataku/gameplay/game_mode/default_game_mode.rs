use crate::prelude::*;


// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode;

#[async_trait]
impl GameMode for NoMode {
    async fn new(_:&Beatmap, _:bool) -> Result<Self, TatakuError> where Self: Sized {Ok(Self {})}

    async fn handle_replay_frame(&mut self, _:ReplayFrame, _:f32, _:&mut IngameManager) {}
    async fn update(&mut self, _:&mut IngameManager, _: f32) -> Vec<ReplayFrame> { Vec::new() }
    async fn draw(&mut self, _:&mut IngameManager, _: &mut RenderableCollection) {}
    fn skip_intro(&mut self, _: &mut IngameManager) {}
    async fn reset(&mut self, _:&Beatmap) {}
    async fn window_size_changed(&mut self, _: Arc<WindowSize>) {}
    async fn fit_to_area(&mut self, _:Vector2, _:Vector2) {}
    async fn force_update_settings(&mut self, _: &Settings) {}
    
    async fn reload_skin(&mut self) {}
    async fn apply_mods(&mut self, _: Arc<ModManager>) {}

    
    async fn beat_happened(&mut self, _pulse_length: f32) {}
    async fn kiai_changed(&mut self, _is_kiai: bool) {}
}

#[async_trait]
impl GameModeInput for NoMode {
    async fn key_down(&mut self, _:Key) -> Option<ReplayFrame> { None }
    async fn key_up(&mut self, _:Key) -> Option<ReplayFrame> { None }
}

impl GameModeProperties for NoMode {
    fn playmode(&self) -> PlayMode {"none".to_owned()}
    fn end_time(&self) -> f32 {0.0}
    
    // fn combo_bounds(&self) -> Rectangle {SimpleRectangle::new(Vector2::ZERO, Vector2::ZERO)}
    fn timing_bar_things(&self) -> Vec<(f32,Color)> { Vec::new() }
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> { Vec::new() }
    // fn score_hit_string(_hit:&ScoreHit) -> String where Self: Sized { String::new() }
}
