use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct DropdownElement {
    #[serde(rename = "@options_path")] options_path: engine::VariablePathResolver,
    // #[serde(rename = "@options_display_path", default)] options_display_path: Option<ArcStr>,
    #[serde(rename = "@selected_path")] selected_path: engine::VariablePathResolver,

    #[serde(rename = "@placeholder", default)] placeholder_attribute: Option<ArcStr>,
    #[serde(default)] placeholder: Option<Wrapped<Vec<BuildableText>>>,

    #[serde(rename = "onSelect")] on_select: Wrapped<Vec<BuildableAction>>,
}
impl DropdownElement {
    fn placeholder(&self) -> widgets::WidgetText {
        let placeholder = self.placeholder.clone()
            .map(|p| p.inner)
            .or(self.placeholder_attribute.clone()
                .map(|str| vec![BuildableText::Text(str)])
            )
            .unwrap_or_default();

        // todo: static text
        widgets::WidgetText::Custom {
            custom: placeholder,
            cached: String::new()
        }
    }
}
impl DropdownElement {
    pub fn build(&self) -> widgets::Dropdown {
        widgets::Dropdown::new(
            self.options_path.clone().into(),
            self.selected_path.clone().into(),
            self.on_select.inner.clone().into(),
            self.placeholder()
        )
    }
}
