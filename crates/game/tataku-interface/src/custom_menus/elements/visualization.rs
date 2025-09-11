use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualizationElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "$value", default)] visualization: VisualizationType,
}
impl CustomElement for VisualizationElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        let vis = match self.visualization {
            VisualizationType::MenuVisualization => MenuVisualization::new(),
            // _ => { 
            //     warn!("Unknown visualization id: {}", self.visualization); 
            //     return EmptyWidget::new_boxed();
            // },
        };

        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "visualization",
            self.id.clone(),
            self.class_list.clone(),
            widgets::VisualizationWidget::new(vis).boxed()
        )
    }
}

#[derive(Deserialize)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[serde(rename_all="kebab-case")]
pub enum VisualizationType {
    #[default]
    MenuVisualization,
}
