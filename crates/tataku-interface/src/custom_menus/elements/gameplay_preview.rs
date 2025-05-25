use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct GameplayPreviewElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,
    #[serde(rename = "@visualization", default)] visualization: Option<String>,
    
    #[serde(rename = "@blur", default)] blur: f32,
}
impl CustomElement for GameplayPreviewElement {
    fn build(&self) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "gameplayPreview",
            self.id.clone(),
            self.class_list.clone(),
            GameplayPreview::new(
                true, 
                true, 
            )
            .blur(self.blur)
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
