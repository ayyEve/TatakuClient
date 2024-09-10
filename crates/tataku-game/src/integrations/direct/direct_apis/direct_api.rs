use crate::prelude::*;

#[async_trait]
pub trait DirectApi: Send+Sync {
    fn api_name(&self) -> &'static str;
    fn supported_modes(&self) -> Vec<String>;
    async fn do_search(&mut self, search_params:SearchParams, settings: &Settings) -> Vec<Arc<dyn DirectDownloadable>>;

    // TODO: 
    // fn get_search_capabilities(&self) -> SearchCapabilities;
}
