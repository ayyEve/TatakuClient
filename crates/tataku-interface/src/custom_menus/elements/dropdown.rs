use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct DropdownElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,

    #[serde(rename = "@options_path")] options_path: String,
    #[serde(rename = "@options_display_path", default)] options_display_path: Option<String>,
    #[serde(rename = "@selected_path")] selected_path: String,

    #[serde(rename = "@placeholder", default)] placeholder: Option<String>,

    #[serde(alias = "onSelect")] on_select: BuildableActionTag,
}
impl CustomElement for DropdownElement {
    fn build(&self, _shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "dropdown",
            self.id.clone(),
            self.class_list.clone(),
            Dropdown::new(
                self.options_path.clone(),
                self.selected_path.clone(),
                self.on_select.action.clone(),
            )
            .chain_maybe(self.placeholder.clone(), |d, p| d.placeholder(p))
            // .font_size_maybe(font_size)
            // .chain_maybe(font.as_ref().and_then(map_font), |s, font| s.font(font))
            
            // .style(taffy_style)
            .boxed()
        )
    }
}
