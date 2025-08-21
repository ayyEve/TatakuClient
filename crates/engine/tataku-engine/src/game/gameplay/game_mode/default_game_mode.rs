use crate::prelude::*;
#[cfg(feature = "graphics")] use tataku_graphics::prelude::*;


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
    #[cfg(feature="gameplay")] 
    fn skip_intro(&mut self, _: f32) -> Option<f32> { None }

    fn reset(&mut self, _: &Beatmap) {}
    fn force_update_settings(&mut self, _: &Settings) {}
    fn handle_gameplay_event(&mut self, _: GameplayEvent) {}
    
    #[cfg(feature="graphics")]
    fn reload_skin(&mut self, _: &str, _: &mut dyn SkinProvider) -> TextureSource { 
        TextureSource::Raw 
    }

    #[cfg(feature="graphics")] 
    fn get_playfield(&self) -> PlayfieldNonsense { PlayfieldNonsense::default() }
    fn properties(&self, _: &TimingPointHelper) -> GameModeProperties { GameModeProperties::default() }

    #[cfg(feature="gameplay")] 
    fn handle_input(&mut self, _input: InputEvent) -> Option<ReplayAction> { None }
}
