use crate::prelude::*;


#[async_trait]
pub trait GameMode: GameModeInput + GameModeInfo + Send + Sync {
    async fn new(beatmap:&Beatmap, diff_calc_only: bool) -> Result<Self, TatakuError> where Self: Sized;

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager);

    async fn update(&mut self, manager:&mut IngameManager, time: f32);
    async fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut Vec<Box<dyn Renderable>>);

    fn apply_auto(&mut self, settings: &BackgroundGameSettings);
    fn skip_intro(&mut self, manager: &mut IngameManager);
    fn pause(&mut self, _manager:&mut IngameManager) {}
    fn unpause(&mut self, _manager:&mut IngameManager) {}
    async fn reset(&mut self, beatmap:&Beatmap);
}
impl Default for Box<dyn GameMode> {
    fn default() -> Self {
        Box::new(NoMode::default())
    }
}
