use crate::prelude::*;


// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode;
impl GameMode for NoMode {
    fn new(
        _: &Beatmap, 
        _: bool, 
        _: &Settings
    ) -> Result<Self, TatakuError> where Self: Sized {
        Ok(Self {})
    }

    fn handle_replay_frame(&mut self, _: ReplayFrame, _: &mut GameplayUpdateShell) {}
    fn update(&mut self, _: &mut GameplayUpdateShell) { }
    #[cfg(feature="graphics")]
    fn draw(&mut self, _: GameplayDrawShell, _: &mut RenderableCollection) {}
    fn skip_intro(&mut self, _: f32) -> Option<f32> { None }
    fn reset(&mut self, _: &Beatmap) {}
    #[cfg(feature="graphics")]
    fn set_bounds(&mut self, _: Bounds, _: bool) {}
    fn force_update_settings(&mut self, _: &Settings) {}
    
    #[cfg(feature="graphics")]
    fn reload_skin(&mut self, _: &str, _: &mut dyn SkinProvider) -> TextureSource { 
        TextureSource::Raw 
    }
    fn apply_mods(&mut self, _: Arc<ModManager>) {}

    
    fn beat_happened(&mut self, _pulse_length: f32) {}
    fn kiai_changed(&mut self, _is_kiai: bool) {}


    fn get_playfield(&self) -> PlayfieldNonsense { PlayfieldNonsense::default() }
    fn properties(&self, _: &TimingPointHelper) -> GameModeProperties { GameModeProperties::default() }

    fn handle_input(&mut self, _input: InputEvent) -> Option<ReplayAction> { None }
}
