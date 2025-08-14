use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ListElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@list_variable", alias = "@list")] list_var: ArcStr,
    #[serde(rename = "@variable")] variable: ArcStr,
    #[serde(rename = "@scrollable", alias = "@scroll", default)] scrollable: bool,
    
    #[serde(alias = "$value")] element: Element,
}
impl CustomElement for ListElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "list",
            self.id.clone(),
            self.class_list.clone(),
            Container::new(Vec::new())
                .make_programmatic(ProgrammaticListData::new(
                    self.element.clone(),
                    self.list_var.clone(),
                    self.variable.clone(),
                ))
                .scrollable(self.scrollable)
                .drag_scroll(self.scrollable)
                // .style(taffy_style)
                // .vertical_overflow(taffy::Overflow::Clip)
                .boxed()
        )
    }
}
