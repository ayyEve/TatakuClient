use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DropdownElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@options_path")] options_path: ArcStr,
    // #[serde(rename = "@options_display_path", default)] options_display_path: Option<ArcStr>,
    #[serde(rename = "@selected_path")] selected_path: ArcStr,

    #[serde(alias = "@placeholder", default)] placeholder: BuildableText,

    #[serde(alias = "onSelect")] on_select: BuildableAction,
}
impl DropdownElement {
    fn placeholder(&self) -> DropdownPlaceholder {
        match self.placeholder.clone() {
            BuildableText::Text(t) | BuildableText::Locale(t) => DropdownPlaceholder::Static(t),
            buildable => DropdownPlaceholder::Buildable { buildable, cache: String::new() }
        }
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
                self.on_select.clone(),
                self.placeholder()
            )
            .boxed()
        )
    }
}
