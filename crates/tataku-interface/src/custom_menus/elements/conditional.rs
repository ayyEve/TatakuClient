use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct ConditionalElement {
    #[serde(rename = "@id", default)] id: Option<String>,
    #[serde(rename = "@class", default)] class_list: ClassList,

    /// unparsed style string, parsed when the element is built
    #[serde(rename = "@style", default)] style: String,

    #[serde(rename = "@condition", alias = "@cond", default)] condition: String,
    #[serde(rename = "false", default)] if_false: Option<ElementTag>,

    #[serde(rename = "true", default)] if_true_specified: Option<ElementTag>,
    #[serde(rename = "$value", default)] if_true_body: Option<Element>,
}
impl ConditionalElement {
    fn if_true(&self) -> Option<&Element> {
        self.if_true_specified
            .as_ref()
            .map(|i| &i.element)
            .or(self.if_true_body.as_ref())
    }
}

impl CustomElement for ConditionalElement {
    fn build(&self, shell: &mut ElementBuildShell<'_>) -> Box<dyn Widget> {
        let Some(if_true) = self.if_true() else {
            let name = self.id.as_ref()
                .map(|i| format!("id: {i}"))
                .unwrap_or_else(|| format!("cond: {}", self.condition));

            error!("Conditional Element ({name}) does not have an element for when true!");
            return EmptyWidget::new_boxed();
        };

        WidgetContainer::new_boxed(
            self.style.clone(),
            "conditional",
            self.id.clone(),
            self.class_list.clone(),
            ConditionalWidget::new(
                if_true.build(shell),
                self.if_false.as_ref().map(|i| i.build(shell)),
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
            id: Some("cond123".to_owned()),
            class_list: "thing1 thing2".into(),
            condition: "path.to.thing.is_true".to_owned(), 
            if_true_body: Some(TextElement {
                text: BuildableTextInner::Text("hi mom".to_owned()).into(),
                ..Default::default()
            }.into()),
            if_true_specified: None,
            if_false: Some(ElementTag::new(TextElement {
                text: BuildableTextInner::Text("bye mom".to_owned()).into(),
                ..Default::default()
            })),
            ..Default::default()
        }
    )
}
