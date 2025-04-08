use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ButtonElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,
    #[serde(rename = "@active_if", default)] active_cond: Option<BuildableCondition>,
    
    action: BuildableActionTag,
    element: ElementTag,
}
impl CustomElement for ButtonElement {
    fn build(&self, shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "button",
            self.id.clone(),
            self.class_list.clone(),
            Button::new(self.element.element.build(shell))
                .on_press(self.action.action.clone())
                .boxed()
        )
    }
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
            action: BuildableActionTag {
                action: BuildableAction::Song { 
                    action: BuildableSongAction::Play 
                }
            }, 
            element: ElementTag { element: Element::Text(Box::new(TextElement {
                text: BuildableTextInner::Text("hi mom".to_owned()).into(),
                ..Default::default()
            })) } ,
            ..Default::default()
        }
    )
}
