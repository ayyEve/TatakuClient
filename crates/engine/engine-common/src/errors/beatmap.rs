#[derive(Clone, Debug)]
pub enum BeatmapError {
    /// The beatmap file failed to parse
    InvalidFile,

    /// The beatmap can't be used for the playmode
    UnsupportedMode,

    /// Tataku doesn't support this format
    UnsupportedBeatmap,

    /// There are no timing points in this beatmap
    NoTimingPoints,

    /// There are no notes in this beatmap
    NoNotes,

    /// The beatmap couldn't be found in the beatmap set
    NotFoundInSet,
}
