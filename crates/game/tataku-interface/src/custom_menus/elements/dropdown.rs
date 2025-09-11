use crate::prelude::*;
use ui::widget::Widget;
use widgets::DropdownPlaceholder;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DropdownElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@options_path")] options_path: ArcStr,
    // #[serde(rename = "@options_display_path", default)] options_display_path: Option<ArcStr>,
    #[serde(rename = "@selected_path")] selected_path: ArcStr,

    #[serde(rename = "@placeholder", default)] placeholder_attribute: Option<ArcStr>,
    #[serde(default)] placeholder: Option<Wrapped<BuildableText>>,

    #[serde(rename = "onSelect")] on_select: Wrapped<Vec<BuildableAction>>,
}
impl DropdownElement {
    fn placeholder(&self) -> DropdownPlaceholder {
        let placeholder = self.placeholder.clone()
            .map(|p| p.inner)
            .or(self.placeholder_attribute.clone()
                .map(BuildableText::Text)
            )
            .unwrap_or_default();

        match placeholder.clone() {
            BuildableText::Text(t) | BuildableText::Locale(t) => DropdownPlaceholder::Static(t),
            buildable => DropdownPlaceholder::Buildable { buildable, cache: String::new() }
        }
    }
}
impl CustomElement for DropdownElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "dropdown",
            self.id.clone(),
            self.class_list.clone(),
            widgets::Dropdown::new(
                self.options_path.clone(),
                self.selected_path.clone(),
                self.on_select.inner.clone(),
                self.placeholder()
            )
            .boxed()
        )
    }
}
