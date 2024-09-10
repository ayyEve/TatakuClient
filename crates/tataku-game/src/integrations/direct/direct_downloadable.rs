use crate::prelude::*;

/// this item will always be in an arc
/// so nothing will be directly mutable
pub trait DirectDownloadable: Send + Sync {
    /// perform the download
    fn download(&self);

    // get if this item is downloading
    fn is_downloading(&self) -> bool;

    // get the download progress data for this item
    fn get_download_progress(&self) -> &Arc<RwLock<DownloadProgress>>;

    /// get a link to the preview mp3
    /// returns none if not applicable for this api
    fn audio_preview(&self) -> Option<String>;

    /// filename for this downloadable
    fn filename(&self) -> String;

    fn title(&self) -> String;
    fn artist(&self) -> String;
    fn creator(&self) -> String;
}