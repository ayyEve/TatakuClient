use crate::prelude::*;

#[derive(Reflect)]
#[derive(Clone, Debug2, Default)]
pub struct BeatmapSettings {
    #[debug(skip)] 
    #[reflect(rename="buildable")]
    provider: BuildableSettingsProvider,

    beatmap: BeatmapPreferences,
    playmode: BeatmapPlaymodePreferences,
}
impl BeatmapSettings {
    pub fn new(
        beatmap: BeatmapPreferences, 
        playmode: BeatmapPlaymodePreferences,
        values: &mut dyn Reflect,
        prefix: impl ToString,
    ) -> Self {
        let prefix = prefix.to_string();
        let mut builder = SettingsBuilder::new(
            values,
            "Beatmap Settings"
        );
        builder.add_category("", None::<&str>);

        beatmap.create_provider(prefix.clone() + ".beatmap", &mut builder);
        playmode.create_provider(prefix + ".playmode", &mut builder);

        let provider = builder.done();
        println!("{provider:?}");

        Self {
            provider,
            beatmap,
            playmode
        }
    }
}
