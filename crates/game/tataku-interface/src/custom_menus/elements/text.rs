use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextElement {
    #[serde(rename="$value")] pub text: Vec<BuildableText>,
}
#[cfg(feature="graphics")]
impl TextElement {
    pub fn build(&self) -> widgets::Text {
        let mut text = self.text.clone();

        // Trim any literal texts in this element so you can
        // lay them out nicer in xml
        if let Some(BuildableText::Text(first)) = text.first_mut() {
            *first = first.trim_start().into();
        }

        if let Some(BuildableText::Text(last)) = text.last_mut() {
            *last = last.trim_end().into();
        }

        widgets::Text::new(
            widgets::WidgetText::from_buildable_iter(text)
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
