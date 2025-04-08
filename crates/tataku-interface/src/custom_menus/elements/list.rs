use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ListElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,

    #[serde(rename = "@list_variable", alias = "@list")] list_var: String,
    #[serde(rename = "@variable")] variable: String,
    #[serde(rename = "@scrollable", alias = "@scroll", default)] scrollable: bool,
    
    #[serde(alias = "$value")] element: Element,
}
impl CustomElement for ListElement {
    fn build(&self, _shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
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
