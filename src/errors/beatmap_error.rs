

#[derive(Clone, Debug)]
pub enum BeatmapError {
    InvalidFile,
    UnsupportedMode,
    NoTimingPoints,
}