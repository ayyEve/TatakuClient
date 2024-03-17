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
        map.set("path", &beatmap.file_path);
        map.set("display_mode", gamemode_display_name(&beatmap.mode).to_owned());
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