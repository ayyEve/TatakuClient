use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ButtonElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,
    #[serde(rename = "@active_if", default)] active_cond: Option<BuildableCondition>,
    
    #[serde(alias="action")]
    actions: Vec<ClickAction>,
    element: ElementTag,
}
impl CustomElement for ButtonElement {
    fn build(&self) -> Box<dyn Widget> {
        let mut actions = self.actions
            .iter()
            .map(|i| (i.button, i.inner.clone()))
            .collect::<HashMap<MouseButton2, BuildableAction>>();

        WidgetContainer::new_boxed(
            self.style.clone(),
            "button",
            self.id.clone(),
            self.class_list.clone(),
            Button::new(self.element.element.build())
                .on_press_left_maybe(actions.remove(&MouseButton2::Left))
                .on_press_middle_maybe(actions.remove(&MouseButton2::Middle))
                .on_press_right_maybe(actions.remove(&MouseButton2::Right))
                .boxed()
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
struct ClickAction {
    #[serde(rename="$value")] inner: BuildableAction,
    #[serde(rename="@button", default)] button: MouseButton2,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
enum MouseButton2 {
    #[default]
    Left,
    Middle,
    Right
}


#[test]
fn test() {
    use quick_xml::de::from_str;

    assert_eq!(
        from_str::<ButtonElement>(r#"
            <button id="button123" class="thing1 thing2">
                <action> <song> <play/> </song> </action>
                
                <element> <text> hi mom </text> </element>
            </button>
        "#).unwrap(),
        ButtonElement {
            id: Some("button123".to_owned()),
            class_list: "thing1 thing2".into(),
            actions: vec![
                ClickAction {
                    button: MouseButton2::Left,
                    inner: BuildableAction::Song { 
                        action: BuildableSongAction::Play 
                    },
                }
            ], 
            element: ElementTag { element: Element::Text(Box::new(TextElement {
                text: BuildableText::Text { text: "hi mom".to_owned() },
                ..Default::default()
            })) } ,
            ..Default::default()
        }
    );
}
