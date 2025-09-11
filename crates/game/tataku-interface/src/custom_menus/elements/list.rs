use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListElement {
    #[serde(rename = "@list_var", alias = "@list")] list_var: ArcStr,
    #[serde(rename = "@variable", alias="@var")] var: ArcStr,
    #[serde(rename = "@scrollable", alias = "@scroll", default)] scrollable: bool,

    #[serde(rename = "$value")] element: Element,
}
impl ListElement {
    pub fn build(&self) -> widgets::Container {
        widgets::Container::new(Vec::new())
            .make_programmatic(widgets::ProgrammaticListData::new(
                self.element.clone(),
                self.list_var.clone(),
                self.var.clone(),
            ))
            .scrollable(self.scrollable)
            .drag_scroll(self.scrollable)
            // .style(taffy_style)
            // .vertical_overflow(taffy::Overflow::Clip)
    }
}
