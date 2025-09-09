use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameplayPreviewElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,
}
impl CustomElement for GameplayPreviewElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "gameplayPreview",
            self.id.clone(),
            self.class_list.clone(),
            GameplayPreview::new()
            .boxed()
        )
    }
}
