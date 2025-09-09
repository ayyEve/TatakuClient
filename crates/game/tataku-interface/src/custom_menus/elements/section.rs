use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SectionElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,
    #[serde(rename = "$value")] children: Vec<Element>,
}
impl CustomElement for SectionElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "section",
            self.id.clone(),
            self.class_list.clone(),
            Container::new(self
                .children
                .iter()
                .map(|e| e.build())
                .collect()
            ).boxed()
        )
    }
}
