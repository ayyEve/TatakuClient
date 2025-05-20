use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]   
#[derive(Deserialize)]
pub struct TextElement {
    #[serde(rename = "@id", default)] pub id: Option<String>,
    #[serde(rename = "@class", default)] pub class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] pub style: String,
    #[serde(rename = "$value")] pub text: BuildableText,
}
impl CustomElement for TextElement {
    fn build(&self) -> Box<dyn Widget> {
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
                <text text="hi mom"/>
                <text text="hi dad"/>
            </list>
        </text>
    "#;

    assert_eq!(
        quick_xml::de::from_str::<TextElement>(xml).unwrap(), 
        
        TextElement { 
            id: Some("hi".to_owned()), 
            class_list: "thing1 thing2".into(), 
            style: String::new(), 
            text: BuildableText::List {
                join: String::new(),
                list: vec![ 
                    BuildableText::Text { text: "hi mom".to_owned() },
                    BuildableText::Text { text: "hi dad".to_owned() },
                ]
            }
        }
    );
}

