use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColumnElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,
    #[serde(rename = "$value")] children: Vec<Element>,
}
impl CustomElement for ColumnElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        let mut classes = self.class_list.clone();
        classes.push("column");
        
        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "column",
            self.id.clone(),
            classes,
            widgets::Container::new(self.children.iter().map(|e| e.build()).collect()).boxed()
        )
    }
}
