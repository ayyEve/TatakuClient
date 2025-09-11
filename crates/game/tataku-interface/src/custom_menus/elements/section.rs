use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SectionElement {
    #[serde(rename = "$value")] children: Vec<Element>,
}
impl SectionElement {
    pub fn build(&self) -> widgets::Container {
        widgets::Container::new(self
            .children
            .iter()
            .map(|e| e.build().boxed())
            .collect()
        )
    }
}
