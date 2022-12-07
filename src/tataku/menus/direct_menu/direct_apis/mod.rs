use super::prelude::*;

mod osu_direct; pub use osu_direct::*;
mod quaver_direct; pub use quaver_direct::*;

#[async_trait]
pub trait DirectApi: Send+Sync {
    fn api_name(&self) -> &'static str;
    fn supported_modes(&self) -> Vec<PlayMode>;
    async fn do_search(&mut self, search_params:SearchParams) -> Vec<Arc<dyn DirectDownloadable>>;

    // TODO: 
    // fn get_search_capabilities(&self) -> SearchCapabilities;
}
