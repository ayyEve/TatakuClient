use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionalElement {
    #[serde(rename = "@condition", alias = "@cond", default)] condition: ArcStr,

    #[serde(rename = "$value", default)] if_true: Element,
    #[serde(rename = "false", alias="else", default)] if_false: Option<Wrapped<Element>>,
}

impl ConditionalElement {
    pub fn build(&self) -> widgets::ConditionalWidget {
        widgets::ConditionalWidget::new(
            self.if_true.build().boxed(),
            self.if_false.as_ref().map(|i| i.inner.build().boxed()),
            BuildableCondition::Unbuilt(self.condition.clone())
        )
    }
}

// #[test]
// fn test() {
//     use quick_xml::de::from_str;

//     assert_eq!(
//         from_str::<ConditionalElement>(r#"
//             <button id = "cond123" cond="path.to.thing.is_true" class="thing1 thing2">
//                 <true> <text> hi mom </text> </true>
//                 <false> <text> bye mom </text> </false>
//             </button>
//         "#).unwrap(),
//         ConditionalElement {
//             id: Some("cond123".into()),
//             class_list: "thing1 thing2".into(),
//             condition: "path.to.thing.is_true".into(),
//             if_true: TextElement {
//                 text: BuildableText::Text("hi mom".into()),
//                 ..Default::default()
//             }.into(),
//             if_false: Some(ElementTag::new(TextElement {
//                 text: BuildableText::Text("bye mom"),
//                 ..Default::default()
//             })),
//             ..Default::default()
//         }
//     );
// }
