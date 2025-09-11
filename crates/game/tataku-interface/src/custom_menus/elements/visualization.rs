use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualizationElement {
    #[serde(rename = "$value", default)] visualization: VisualizationType,
}
impl VisualizationElement {
    pub fn build(&self) -> widgets::VisualizationWidget {
        let vis = match self.visualization {
            VisualizationType::MenuVisualization => MenuVisualization::new(),
        };

        widgets::VisualizationWidget::new(vis)
    }
}

#[derive(Deserialize)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[serde(rename_all="kebab-case")]
pub enum VisualizationType {
    #[default]
    MenuVisualization,
}
