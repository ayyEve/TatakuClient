use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameplayPreviewElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: ArcStr,
    #[serde(rename = "@visualization", default)] visualization: Option<ArcStr>,
    
    #[serde(rename = "@blur", default)] blur: f32,
    #[serde(rename = "@blur_type", default)] blur_type: CssBlurType,
}
impl CustomElement for GameplayPreviewElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "gameplayPreview",
            self.id.clone(),
            self.class_list.clone(),
            GameplayPreview::new()
            .blur(self.blur_type.into_blur(self.blur))
            .visualization(if let Some(vis) = &self.visualization {
                match &**vis {
                    "menu_visualization" => Some(MenuVisualization::new()),
                    _ => { warn!("Unknown gameplay visualization: {vis}"); None },
                }
            } else { None })
            .boxed()
        )
    }
}
