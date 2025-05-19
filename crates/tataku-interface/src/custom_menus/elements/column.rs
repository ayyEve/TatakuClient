use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ColumnElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,
    #[serde(alias = "$value")] children: Vec<Element>,
}

impl CustomElement for ColumnElement {
    fn build(&self) -> Box<dyn Widget> {
        let mut classes = self.class_list.clone();
        classes.push("column");
        
        WidgetContainer::new_boxed(
            self.style.clone(),
            "column",
            self.id.clone(),
            classes,
            Container::new(self.children.iter().map(|e| e.build()).collect()).boxed()
        )
    }
}
