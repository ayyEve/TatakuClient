use crate::prelude::*;

/// Only showes one item at a time
#[derive(Clone, Debug)]
#[derive(Deserialize)]
pub struct TabbedElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: String,

    #[serde(rename = "@name")] name: String,
    // #[serde(rename = "@name", default)] name_attribute: Option<String>,
    // #[serde(rename = "name", default)] name_tag: Option<BuildableTextTag>,
    #[serde(rename = "$value")] tabs: BuildableTabProvider,
}
impl CustomElement for TabbedElement {
    fn build(&self) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "tabbed",
            self.id.clone(),
            self.class_list.clone(),
                TabbedWidget::new(
                    self.name.clone(),
                    self.tabs.build(),
                )
                .boxed()
        )
    }
}



#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
struct BuildableTab {
    #[serde(rename="@name")] name: String,
    // #[serde(rename="@name", default)] pub name_attribute: Option<BuildableText>,
    // #[serde(rename="name", default)] pub name_tag: Option<BuildableTextTag>,
    #[serde(alias="$value")] element: Element,
}


#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
enum BuildableTabProvider {
    Static {
        #[serde(alias="$value")] 
        tabs: Vec<BuildableTab>,
    },

    Programmatic {
        /// list to get values from
        #[serde(rename="@list")] list: String,

        /// variable to put the values in
        #[serde(rename="@variable")] variable: String,

        /// where to get the name of the tab from
        #[serde(rename="@name")] name_path: String,

        /// the tab template
        #[serde(rename="$value")] template: Element,
    }
}
impl BuildableTabProvider {
    pub fn build(
        &self, 
        // values: &dyn Reflect,
    ) -> TabProvider {
        match self {
            Self::Static { 
                tabs 
            } => TabProvider::Static(
                tabs.iter()
                    .map(|tab| Tab::new(tab.name.clone(), tab.element.build()))
                    .collect(),

                // tabs.iter()
                //     .filter_map(|i| i.name_tag
                //         .as_ref()
                //         .map(|i| &i.value)
                //         .or(i.name_attribute.as_ref())
                //         .map(|name| (name.clone(), &i.element))
                //     )
                //     .filter_map(|(mut i, a)| i.compute().map(|_| (i, a)).ok())
                //     .map(|(name, ele)| Tab::new(name.to_string(values), ele.build()))
                //     .collect()
            ),

            Self::Programmatic { 
                list, 
                variable, 
                name_path, 
                template 
            } => TabProvider::Programmatic { 
                list: list.clone(), 
                variable: variable.clone(), 
                name_path: name_path.clone(), 
                template: template.clone(), 
                cached: Vec::new(),
            },
        }
    }
}
