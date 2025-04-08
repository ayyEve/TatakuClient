use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]   
#[derive(Deserialize)]
pub struct TextElement {
    #[serde(rename = "@id", default)] pub id: Option<String>,
    #[serde(rename = "@class", default)] pub class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] pub style: String,
    #[serde(alias = "$value", alias = "$text")] pub text: BuildableText,
}

impl CustomElement for TextElement {
    fn build(&self, _shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "text",
            self.id.clone(),
            self.class_list.clone(),
            TextWidget::new(
                self.text.clone()
            )
            .boxed()
        )
    }

}


#[test]
fn test() {
    let xml = r#"
        <text id="hi" class="thing1 thing2">
            <list>
                <text>hi mom</text>
                <text>hi dad</text>
            </list>
        </text>
    "#;

    assert_eq!(
        quick_xml::de::from_str::<TextElement>(xml).unwrap(), 
        
        TextElement { 
            id: Some("hi".to_owned()), 
            class_list: "thing1 thing2".into(), 
            style: String::new(), 
            text: BuildableText {
                join: None,
                text: vec![
                    BuildableTextInner::Text("hi mom".to_owned()),
                    BuildableTextInner::Text("hi dad".to_owned()),
                ]
            }
        }
    )
}

