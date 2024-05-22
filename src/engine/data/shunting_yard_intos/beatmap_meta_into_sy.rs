use crate::prelude::*;

impl From<&BeatmapMeta> for TatakuValue {
    fn from(beatmap: &BeatmapMeta) -> Self {
        let mut map = ValueCollectionMapHelper::default();

        map.set("artist", TatakuVariable::new(&beatmap.artist));
        map.set("title", TatakuVariable::new(&beatmap.title));
        map.set("creator", TatakuVariable::new(&beatmap.creator));
        map.set("version", TatakuVariable::new(&beatmap.version));
        map.set("playmode", TatakuVariable::new(&beatmap.mode).display(gamemode_display_name(&beatmap.mode)));
        map.set("game", TatakuVariable::new(format!("{:?}", beatmap.beatmap_type)));
        map.set("hash", TatakuVariable::new(&beatmap.beatmap_hash.to_string()));
        map.set("audio_path", TatakuVariable::new(&beatmap.audio_filename));
        map.set("preview_time", TatakuVariable::new(beatmap.audio_preview));
        map.set("image_filename", TatakuVariable::new(&beatmap.image_filename));
        map.set("path", TatakuVariable::new(&beatmap.file_path));
        // map.set("display_mode", TatakuVariable::new_readonly());
        map.set("beatmap_type", TatakuVariable::new(&beatmap.beatmap_type));
        map.finish()
    }
}
impl TryInto<BeatmapMeta> for &TatakuValue {
    type Error = String;
    fn try_into(self) -> Result<BeatmapMeta, Self::Error> {
        let TatakuValue::Map(map) = self else { return Err(format!("not a map")) };
        Ok(BeatmapMeta {
            artist: map.get("artist").ok_or_else(|| "no artist".to_owned())?.as_string(),
            title: map.get("title").ok_or_else(|| "no title".to_owned())?.as_string(),

            creator: map.get("creator").ok_or_else(|| "no creator".to_owned())?.as_string(),
            version: map.get("version").ok_or_else(|| "no version".to_owned())?.as_string(),
            mode: map.get("playmode").ok_or_else(|| "no playmode".to_owned())?.as_string(),
            beatmap_type: map.get("game").ok_or_else(|| "no game".to_owned())?.as_ref().try_into().map_err(|e:String| e.to_string())?,
            beatmap_hash: map.get("hash").ok_or_else(|| "no hash".to_owned())?.as_ref().try_into().map_err(|e:String| e.to_string())?,
            audio_filename: map.get("audio_filename").ok_or_else(|| "no audio_filename".to_owned())?.as_string(),
            audio_preview: map.get("preview_time").ok_or_else(|| "no preview_time".to_owned())?.as_f32().map_err(|e| format!("{e:?}"))?,

            file_path: map.get("path").ok_or_else(|| "no path".to_owned())?.as_string(),
            image_filename: map.get("image_filename").ok_or_else(|| "no image_filename".to_owned())?.as_string(),

            ..Default::default()
        })
    }
}

impl From<&BeatmapType> for TatakuValue {
    fn from(value: &BeatmapType) -> Self {
        Self::String(format!("{value:?}"))
    }
}

impl TryInto<BeatmapType> for &TatakuValue {
    type Error = String;
    fn try_into(self) -> Result<BeatmapType, Self::Error> {
        match self {

            TatakuValue::String(s) => {
                match &**s {
                    "Osu" | "osu" => Ok(BeatmapType::Osu),
                    "Quaver" | "quaver" => Ok(BeatmapType::Quaver),
                    "Stepmania" | "stepmania" => Ok(BeatmapType::Stepmania),
                    "Tja" | "tja" => Ok(BeatmapType::Tja),
                    "Utyping" | "u_typing" => Ok(BeatmapType::UTyping),
                    other => Err(format!("invalid BeatmapType str: '{other}'"))
                }
            }

            TatakuValue::U64(n) => {
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
