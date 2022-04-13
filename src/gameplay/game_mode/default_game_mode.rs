use crate::prelude::*;


// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode {}

#[async_trait]
impl GameMode for NoMode {
    async fn new(_:&Beatmap, _:bool) -> Result<Self, TatakuError> where Self: Sized {Ok(Self {})}

    async fn handle_replay_frame(&mut self, _:ReplayFrame, _:f32, _:&mut IngameManager) {}
    async fn update(&mut self, _:&mut IngameManager, _: f32) {}
    async fn draw(&mut self, _:RenderArgs, _:&mut IngameManager, _: &mut Vec<Box<dyn Renderable>>) {}
    fn apply_auto(&mut self, _: &BackgroundGameSettings) {}
    fn skip_intro(&mut self, _: &mut IngameManager) {}
    async fn reset(&mut self, _:&Beatmap) {}
}

#[async_trait]
impl GameModeInput for NoMode {
    async fn key_down(&mut self, _:piston::Key, _:&mut IngameManager) {}
    async fn key_up(&mut self, _:piston::Key, _:&mut IngameManager) {}
}

impl GameModeInfo for NoMode {
    fn playmode(&self) -> PlayMode {"osu".to_owned()}
    fn end_time(&self) -> f32 {0.0}
    
    // fn combo_bounds(&self) -> Rectangle {Rectangle::bounds_only(Vector2::zero(), Vector2::zero())}
    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {(Vec::new(), (0.0, Color::WHITE))}
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {Vec::new()}
    fn score_hit_string(_hit:&ScoreHit) -> String where Self: Sized {String::new()}
}