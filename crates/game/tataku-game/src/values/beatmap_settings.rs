use crate::prelude::*;
use common::reflect::*;

#[cfg(feature="graphics")]
use engine::settings::{
    SettingsBuilder,
    MakeSettingsMenu,
    BuildableSettingsProvider,
};

#[derive(Reflect)]
#[derive(Clone, Debug2, Default)]
pub struct BeatmapSettings {
    #[debug(skip)] 
    #[cfg(feature="graphics")]
    #[reflect(rename="buildable")]
    provider: BuildableSettingsProvider,

    beatmap: engine::data::BeatmapPreferences,
    playmode: engine::data::BeatmapPlaymodePreferences,
}
impl BeatmapSettings {
    pub fn new(
        beatmap: engine::data::BeatmapPreferences, 
        playmode: engine::data::BeatmapPlaymodePreferences,
        values: &mut dyn Reflect,
        prefix: impl Into<String>,
    ) -> Self {
        let prefix = prefix.into();

        #[cfg(feature="graphics")] 
        let provider = {
            let mut builder = SettingsBuilder::new(
                values,
                "Beatmap Settings"
            );
            builder.add_category("", None::<&str>);

            beatmap.create_provider(prefix.clone() + ".beatmap", &mut builder);
            playmode.create_provider(prefix + ".playmode", &mut builder);

            builder.done()
        };

        Self {
            #[cfg(feature="graphics")] provider,
            beatmap,
            playmode
        }
    }
}
