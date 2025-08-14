use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct DropdownElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@options_path")] options_path: ArcStr,
    #[serde(rename = "@options_display_path", default)] options_display_path: Option<ArcStr>,
    #[serde(rename = "@selected_path")] selected_path: ArcStr,

    #[serde(rename = "@placeholder", default)] placeholder_attribute: Option<ArcStr>,
    #[serde(rename = "placeholder", default)] placeholder_tag: Option<BuildableTextTag>,

    #[serde(alias = "onSelect")] on_select: BuildableActionTag,
}
impl DropdownElement {
    fn placeholder(&self) -> Option<DropdownPlaceholder> {
        self
            .placeholder_attribute
            .clone()
            .map(DropdownPlaceholder::Static)
            .or(self.placeholder_tag
                .clone()
                .map(|b| b.value.into())
            )
    }
}
impl CustomElement for DropdownElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "dropdown",
            self.id.clone(),
            self.class_list.clone(),
            Dropdown::new(
                self.options_path.clone(),
                self.selected_path.clone(),
                self.on_select.action.clone(),
                self.placeholder().unwrap_or_default()
            )
            .boxed()
        )
    }
}
