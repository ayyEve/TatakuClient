use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionalElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(rename = "@condition", alias = "@cond", default)] condition: ArcStr,
    #[serde(rename = "false", default)] if_false: Option<ElementTag>,

    #[serde(rename = "true", default)] if_true_tag: Option<ElementTag>,
    #[serde(rename = "$value", default)] if_true: Option<Element>,
}
impl ConditionalElement {
    fn if_true(&self) -> Option<&Element> {
        self.if_true_tag
            .as_ref()
            .map(|i| &i.element)
            .or(self.if_true.as_ref())
    }
}

impl CustomElement for ConditionalElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        let Some(if_true) = self.if_true() else {
            let name = self.id
                .as_ref()
                .map_or_else(
                    || format!("cond: {}", self.condition), 
                    |i| format!("id: {i}")
                );

            error!("Conditional Element ({name}) does not have an element for when true!");
            return EmptyWidget::new_boxed();
        };

        WidgetContainer::new_boxed(
            self.style.clone(),
            "conditional",
            self.id.clone(),
            self.class_list.clone(),
            ConditionalWidget::new(
                if_true.build(),
                self.if_false.as_ref().map(|i| i.build()),
                BuildableCondition::Unbuilt(self.condition.clone())
            )
            .boxed()
        )
    }
}

#[test]
fn test() {
    use quick_xml::de::from_str;

    assert_eq!(
        from_str::<ConditionalElement>(r#"
            <button id = "cond123" cond="path.to.thing.is_true" class="thing1 thing2">
                <true> <text> hi mom </text> </true>
                <false> <text> bye mom </text> </false>
            </button>
        "#).unwrap(),
        ConditionalElement {
            id: Some("cond123".into()),
            class_list: "thing1 thing2".into(),
            condition: "path.to.thing.is_true".into(), 
            if_true: Some(TextElement {
                text: BuildableText::Text { text: "hi mom".into() },
                ..Default::default()
            }.into()),
            if_true_tag: None,
            if_false: Some(ElementTag::new(TextElement {
                text: BuildableText::Text { text: "bye mom".into() },
                ..Default::default()
            })),
            ..Default::default()
        }
    );
}
