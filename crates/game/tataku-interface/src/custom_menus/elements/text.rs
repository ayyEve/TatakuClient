use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextElement {
    #[serde(rename = "@id", default)] pub id: Option<ArcStr>,
    #[serde(rename = "@class", default)] pub class_list: ClassList,
    #[serde(rename = "@style", default)] pub style: ArcStr,

    #[serde(rename="$value")] pub text: Vec<BuildableText>,
}
impl CustomElement for TextElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        let mut text = self.text.clone();

        // Trim any literal texts in this element so you can
        // lay them out nicer in xml
        if let Some(BuildableText::Text(first)) = text.first_mut() {
            *first = first.trim_start().into();
        }

        if let Some(BuildableText::Text(last)) = text.last_mut() {
            *last = last.trim_end().into();
        }

        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "text",
            self.id.clone(),
            self.class_list.clone(),
            widgets::TextWidget::new(
                widgets::WidgetText::from_buildable_iter(text)
            )
            .boxed()
        )
    }
}


// #[test]
// fn test() {
//     let xml = r#"
//         <text id="hi" class="thing1 thing2">
//             <list>
//                 <text text="hi mom"/>
//                 <text text="hi dad"/>
//             </list>
//         </text>
//     "#;

//     assert_eq!(
//         quick_xml::de::from_str::<TextElement>(xml).unwrap(),

//         TextElement {
//             id: Some("hi".into()),
//             class_list: "thing1 thing2".into(),
//             style: ArcStr::default(),
//             text: BuildableText::List {
//                 join: ArcStr::default(),
//                 list: vec![
//                     BuildableText::Text("hi mom".into()),
//                     BuildableText::Text("hi dad".into()),
//                 ]
//             }
//         }
//     );
// }
