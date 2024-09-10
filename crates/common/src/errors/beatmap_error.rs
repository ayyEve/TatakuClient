

#[derive(Clone, Debug)]
pub enum BeatmapError {
    InvalidFile,
    UnsupportedMode,
    UnsupportedBeatmap,
    NoTimingPoints,
    NoNotes,
    NotFoundInSet,
}