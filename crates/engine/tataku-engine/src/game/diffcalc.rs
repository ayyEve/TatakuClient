use crate::*;
pub use gameplay::mode::*;


pub trait DiffCalc: Send + Sync {
    fn new(
        g: &engine::BeatmapMeta,
        settings: &settings::Settings
    ) -> tataku::Result<Self> where Self:Sized;

    fn calc(
        &mut self,
        mods: &gameplay::mods::Mods
    ) -> tataku::Result<DiffCalcSummary>;
}

#[derive(Default)]
#[derive(Serialize)]
pub struct DiffCalcSummary {
    pub diff: f32,
    pub diffs: Vec<f32>,
    pub strains: HashMap<String, Vec<f32>>
}
impl DiffCalcSummary {
    #[allow(unused)]
    pub fn save(&self, path: impl AsRef<Path>) -> tataku::Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
