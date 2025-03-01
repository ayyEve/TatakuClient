use crate::prelude::*;


// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode;

#[async_trait]
impl GameMode for NoMode {
    async fn new(_: &Beatmap, _: bool, _: &Settings) -> Result<Self, TatakuError> where Self: Sized {Ok(Self {})}

    async fn handle_replay_frame<'a>(&mut self, _: ReplayFrame, _: &mut GameplayUpdateShell<'a>) {}
    async fn update<'a>(&mut self, _: &mut GameplayUpdateShell<'a>) { }
    async fn draw<'a>(&mut self, _: GameplayDrawShell<'a>, _: &mut RenderableCollection) {}
    fn skip_intro(&mut self, _: f32) -> Option<f32> { None }
    async fn reset(&mut self, _: &Beatmap) {}
    fn set_bounds(&mut self, _: Bounds, _: bool) {}
    async fn force_update_settings(&mut self, _: &Settings) {}
    
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, _beatmap_folder: &str, _skin_manager: &mut dyn SkinProvider) -> TextureSource { TextureSource::Raw }
    async fn apply_mods(&mut self, _: Arc<ModManager>) {}

    
    async fn beat_happened(&mut self, _pulse_length: f32) {}
    async fn kiai_changed(&mut self, _is_kiai: bool) {}


    fn get_playfield(&self) -> PlayfieldNonsense { Default::default() }
    fn properties(&self) -> GameModeProperties { GameModeProperties::default() }

    async fn handle_input(&mut self, _input: InputEvent) -> Option<ReplayAction> { None }
}
