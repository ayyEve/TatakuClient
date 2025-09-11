use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,
    
    #[serde(alias = "$value")] children: Vec<Element>,
}
impl CustomElement for RowElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        let mut classes = self.class_list.clone();
        classes.push("row");

        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "row",
            self.id.clone(),
            classes,
            widgets::Container::new(
                self.children
                    .iter()
                    .map(|e| e.build())
                    .collect()
                )
                // .style(taffy_style)
                // .flex_direction(taffy::FlexDirection::Row)
                // .vertical_overflow(taffy::Overflow::Clip)
                .boxed()
        )
    }
}
