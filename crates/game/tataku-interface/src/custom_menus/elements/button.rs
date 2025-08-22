use crate::prelude::*;

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
    element: ElementTag,
}
impl CustomElement for ButtonElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        let mut actions = self.actions
            .iter()
            .map(|i| (i.button, i.inner.clone()))
            .collect::<HashMap<MouseButton2, BuildableAction>>();

        WidgetContainer::new_boxed(
            self.style.clone(),
            "button",
            self.id.clone(),
            self.class_list.clone(),
            Button::new(self.element.build())
                .on_press_left_maybe(actions.remove(&MouseButton2::Left))
                .on_press_middle_maybe(actions.remove(&MouseButton2::Middle))
                .on_press_right_maybe(actions.remove(&MouseButton2::Right))
                .active_condition_maybe(self.active_override.clone())
                .boxed()
        )
    }
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
struct ClickAction {
    #[serde(rename="$value")] inner: BuildableAction,
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
            id: Some("button123".into()),
            class_list: "thing1 thing2".into(),
            actions: vec![
                ClickAction {
                    button: MouseButton2::Left,
                    inner: BuildableAction::Song { 
                        action: BuildableSongAction::Play 
                    },
                }
            ], 
            element: ElementTag { inner: Element::Text(Box::new(TextElement {
                text: BuildableText::Text { text: "hi mom".into() },
                ..Default::default()
            })) } ,
            ..Default::default()
        }
    );
}
