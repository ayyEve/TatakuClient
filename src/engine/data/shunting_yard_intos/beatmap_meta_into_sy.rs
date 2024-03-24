use crate::prelude::*;

impl From<&BeatmapMeta> for CustomElementValue {
    fn from(beatmap: &BeatmapMeta) -> Self {
        let mut map = CustomElementMapHelper::default();
        map.set("artist", &beatmap.artist);
        map.set("title", &beatmap.title);
        map.set("creator", &beatmap.creator);
        map.set("version", &beatmap.version);
        map.set("playmode", &beatmap.mode);
        map.set("game", format!("{:?}", beatmap.beatmap_type));
        map.set("hash", &beatmap.beatmap_hash.to_string());
        map.set("audio_path", &beatmap.audio_filename);
        map.set("preview_time", beatmap.audio_preview);
        map.set("image_filename", &beatmap.image_filename);
        map.set("path", &beatmap.file_path);
        map.set("display_mode", gamemode_display_name(&beatmap.mode).to_owned());
        map.set("beatmap_type", &beatmap.beatmap_type);
        map.finish()
    }
}
// impl TryInto<BeatmapMeta> for &CustomElementValue {
//     type Error = String;
//     fn try_into(self) -> Result<BeatmapMeta, Self::Error> {
//         let str = self.as_string();
//         Md5Hash::try_from(str).map_err(|e| format!("{e:?}"))
//     }
// }

impl From<&BeatmapType> for CustomElementValue {
    fn from(value: &BeatmapType) -> Self {
        Self::String(format!("{value:?}"))
    }
}

impl TryInto<BeatmapType> for &CustomElementValue {
    type Error = String;
    fn try_into(self) -> Result<BeatmapType, Self::Error> {
        match self {

            CustomElementValue::String(s) => {
                match &**s {
                    "Osu" | "osu" => Ok(BeatmapType::Osu),
                    "Quaver" | "quaver" => Ok(BeatmapType::Quaver),
                    "Stepmania" | "stepmania" => Ok(BeatmapType::Stepmania),
                    "Tja" | "tja" => Ok(BeatmapType::Tja),
                    "Utyping" | "u_typing" => Ok(BeatmapType::UTyping),
                    other => Err(format!("invalid BeatmapType str: '{other}'"))
                }
            }

            CustomElementValue::U64(n) => {
                match *n {
                    0 => Ok(BeatmapType::Osu),
                    1 => Ok(BeatmapType::Quaver),
                    2 => Ok(BeatmapType::Stepmania),
                    3 => Ok(BeatmapType::Tja),
                    4 => Ok(BeatmapType::UTyping),
                    other => Err(format!("Invalid BeatmapType number: {other}")),
                }
            }

            other => Err(format!("Invalid BeatmapType value: {other:?}"))
        }
    }
}
