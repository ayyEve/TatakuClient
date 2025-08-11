use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct RowElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: String,
    
    #[serde(alias = "$value")] children: Vec<Element>,
}
impl CustomElement for RowElement {
    fn build(&self) -> Box<dyn Widget> {
        let mut classes = self.class_list.clone();
        classes.push("row");

        WidgetContainer::new_boxed(
            self.style.clone(),
            "row",
            self.id.clone(),
            classes,
            Container::new(
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
