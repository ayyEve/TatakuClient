use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]   
pub struct TextElement {
    #[serde(rename = "@id", default)] pub id: Option<ArcStr>,
    #[serde(rename = "@class", default)] pub class_list: ClassList,
    #[serde(rename = "@style", default)] pub style: ArcStr,
    
    #[serde(rename = "$value")] pub text: BuildableText,
}
impl CustomElement for TextElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
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
            id: Some("hi".into()), 
            class_list: "thing1 thing2".into(), 
            style: ArcStr::default(), 
            text: BuildableText::List {
                join: ArcStr::default(),
                list: vec![ 
                    BuildableText::Text { text: "hi mom".into() },
                    BuildableText::Text { text: "hi dad".into() },
                ]
            }
        }
    );
}

