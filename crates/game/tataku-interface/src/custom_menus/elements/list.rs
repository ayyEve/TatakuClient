use crate::prelude::*;
use widgets::ScrollDirection;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListElement {
    #[serde(rename = "@list_var", alias = "@list")] list_var: ArcStr,
    #[serde(rename = "@variable", alias="@var")] var: ArcStr,
    #[serde(rename = "@scroll", alias = "@scrollable", default)] scroll_direction: ScrollDirection,

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
            .scroll_direction(self.scroll_direction)
            .drag_scroll(!matches!(self.scroll_direction, ScrollDirection::None))
            // .style(taffy_style)
            // .vertical_overflow(taffy::Overflow::Clip)
    }
}

