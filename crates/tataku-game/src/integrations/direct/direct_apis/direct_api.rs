use crate::prelude::*;

#[async_trait]
pub trait DirectApi: Send+Sync {

    // TODO: make Cow?
    fn api_name(&self) -> &'static str;

    // TODO: make &[&str]?
    fn supported_modes(&self) -> Vec<String>;
    async fn do_search(&mut self, search_params: SearchParams, settings: &Settings) -> Vec<Arc<dyn DirectDownloadable>>;

    // TODO: 
    // fn get_search_capabilities(&self) -> SearchCapabilities;
}


/// TODO: move this to a struct, with the download fn being a boxed fn that returns the download progress
/// this item will always be in an arc
/// so nothing will be directly mutable
pub trait DirectDownloadable: Send + Sync {
    /// perform the download
    fn download(&self, settings: &Settings);

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
