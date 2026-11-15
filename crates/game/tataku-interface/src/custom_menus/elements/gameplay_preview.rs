use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameplayPreviewElement;

#[cfg(feature="graphics")]
impl GameplayPreviewElement {
    pub fn build(&self) -> widgets::GameplayPreview {
        widgets::GameplayPreview::new()
    }
}
