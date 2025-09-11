use crate::prelude::*;
use ui::widget::Widget;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ButtonElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: ArcStr,
    #[serde(rename = "@active", default)] active_override: Option<BuildableCondition>,
    
    #[serde(alias="action")]
    actions: Vec<ClickAction>,

    #[serde(rename = "$value")]
    element: Element,
}
impl CustomElement for ButtonElement {
    fn build(&self) -> Box<dyn Widget<actions::Action>> {
        let mut left = Vec::new();
        let mut middle = Vec::new();
        let mut right = Vec::new();

        for action in self.actions.iter() {
            let vec = match action.button {
                MouseButton2::Left => &mut left,
                MouseButton2::Middle => &mut middle,
                MouseButton2::Right => &mut right,
            };

            vec.extend(action.inner.iter().cloned());
        }

        widgets::WidgetContainer::new_boxed(
            self.style.clone(),
            "button",
            self.id.clone(),
            self.class_list.clone(),
            widgets::Button::new(self.element.build())
                .on_press_left(left)
                .on_press_middle(middle)
                .on_press_right(right)
                .active_condition_maybe(self.active_override.clone())
                .boxed()
        )
    }
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
struct ClickAction {
    #[serde(rename="$value")] inner: Vec<BuildableAction>,
    #[serde(rename="@button", default)] button: MouseButton2,
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum MouseButton2 {
    #[default]
    Left,
    Middle,
    Right
}


// #[test]
// fn test() {
//     use quick_xml::de::from_str;

//     assert_eq!(
//         from_str::<ButtonElement>(r#"
//             <button id="button123" class="thing1 thing2">
//                 <action> <song> <play/> </song> </action>

//                 <text> hi mom </text>
//             </button>
//         "#).unwrap(),
//         ButtonElement {
//             id: Some("button123".into()),
//             class_list: "thing1 thing2".into(),
//             actions: vec![
//                 ClickAction {
//                     button: MouseButton2::Left,
//                     inner: BuildableAction::Song {
//                         action: BuildableSongAction::Play
//                     },
//                 }
//             ],
//             element: ElementTag { inner: Element::Text(Box::new(TextElement {
//                 text: BuildableText::Text("hi mom".into()),
//                 ..Default::default()
//             })) } ,
//             ..Default::default()
//         }
//     );
// }
