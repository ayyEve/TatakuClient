use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListElement {
    #[serde(rename = "@list_var", alias = "@list")] list_var: ArcStr,
    #[serde(rename = "@variable", alias="@var")] var: ArcStr,
    #[serde(rename = "@scroll", alias = "@scrollable", default)] scroll_direction: XmlScrollDirection,

    #[serde(rename = "$value")] element: Element,
}
#[cfg(feature="graphics")]
impl ListElement {
    pub fn build(&self) -> widgets::Container {
        widgets::Container::new(Vec::new())
            .make_programmatic(widgets::ProgrammaticListData::new(
                self.element.clone(),
                self.list_var.clone(),
                self.var.clone(),
            ))
            .scroll_direction(self.scroll_direction)
            .drag_scroll(!matches!(self.scroll_direction, XmlScrollDirection::None))
            // .style(taffy_style)
            // .vertical_overflow(taffy::Overflow::Clip)
    }
}

#[derive(Deserialize)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[serde(rename_all="kebab-case")]
pub enum XmlScrollDirection {
    #[default]
    None,
    Vertical,
    Horizontal,
    Both,
}

#[cfg(feature="graphics")]
impl From<XmlScrollDirection> for widgets::ScrollDirection {
    fn from(value: XmlScrollDirection) -> Self {
        match value {
            XmlScrollDirection::None => widgets::ScrollDirection::None,
            XmlScrollDirection::Both => widgets::ScrollDirection::Both,
            XmlScrollDirection::Vertical => widgets::ScrollDirection::Vertical,
            XmlScrollDirection::Horizontal => widgets::ScrollDirection::Horizontal,
        }
    }
}