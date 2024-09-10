use crate::prelude::*;


// needed for std::mem::take/swap
#[derive(Default)]
pub struct NoMode;

#[async_trait]
impl GameMode for NoMode {
    async fn new(_:&Beatmap, _:bool, _: &Settings) -> Result<Self, TatakuError> where Self: Sized {Ok(Self {})}

    async fn handle_replay_frame<'a>(&mut self, _: ReplayFrame, _: &mut GameplayStateForUpdate<'a>) {}
    async fn update<'a>(&mut self, _: &mut GameplayStateForUpdate<'a>) { }
    async fn draw<'a>(&mut self, _:GameplayStateForDraw<'a>, _: &mut RenderableCollection) {}
    fn skip_intro(&mut self, _: f32) -> Option<f32> { None }
    async fn reset(&mut self, _:&Beatmap) {}
    async fn window_size_changed(&mut self, _: Arc<WindowSize>) {}
    async fn fit_to_area(&mut self, _:Bounds) {}
    async fn force_update_settings(&mut self, _: &Settings) {}
    
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, _beatmap_folder: &String, _skin_manager: &mut dyn SkinProvider) -> TextureSource { TextureSource::Raw }
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
    fn get_info(&self) -> GameModeInfo {
        GameModeInfo::default()
    }
}

// #[derive(Debug)]
// struct EmptyGamemodeInfo;
// #[async_trait]
// impl GameModeInfo for EmptyGamemodeInfo {
//     fn new() -> Self where Self:Sized { Self }
//     fn id(&self) ->  &'static str { "none" }

//     fn display_name(&self) ->  &'static str { "None" }
//     fn calc_acc(&self, _: &Score) -> f32 { 0.0 }

//     fn get_judgments(&self) -> Vec<HitJudgment> { Vec::new() }

//     fn get_diff_string(&self, _: &BeatmapMetaWithDiff, _: &ModManager) -> String {
//         String::new()
//     }
    
//     async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>> {
//         unimplemented!()
//     }
//     async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>> {
//         unimplemented!()
//     }

// }