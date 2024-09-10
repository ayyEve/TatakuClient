
/// async helper
pub struct LoadingStatus {
    pub stage: LoadingStage,
    pub error: Option<String>,

    pub item_count: usize, // items in the list
    pub items_complete: usize, // items done loading in the list
    pub custom_message: String,

    pub complete: bool,
}
impl LoadingStatus {
    pub fn new(stage: LoadingStage) -> Self {
        Self {
            error: None,
            item_count: 0,
            items_complete: 0,
            stage,
            custom_message: String::new(),

            complete: false
        }
    }

}

#[derive(Clone, Copy, Debug)]
pub enum LoadingStage {
    Difficulties,
    Beatmaps,
    Integrations,
    Fonts,
}
impl LoadingStage {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Difficulties => "Loading difficulties",
            Self::Beatmaps => "Loading beatmaps",
            Self::Integrations => "Initializing integrations",
            Self::Fonts => "Initializing fonts",
        }
    }
}