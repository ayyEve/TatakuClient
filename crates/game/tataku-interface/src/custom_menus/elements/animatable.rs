use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimatableElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,

    #[serde(default)] triggers: Vec<AnimatableTrigger>,
    #[serde(default)] actions: Vec<AnimatableActionEntry>,

    #[serde(rename = "$value")]
    element: Element,
}
impl CustomElement for AnimatableElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "animatable",
            self.id.clone(),
            self.class_list.clone(),
            TransformableWidget::new(
                self.triggers.clone(),
                self.actions.iter()
                    .cloned()
                    .map(|i| (i.id, i.list))
                    .collect(),
                self.element.build()
            )
            .boxed()
        )
    }
}


#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
struct AnimatableActionEntry {
    #[serde(rename = "@id")] id: String,
    #[serde(alias = "$value")] list: Vec<AnimatableAction>
}

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct AnimatableTrigger {
    #[serde(alias = "$value")] pub trigger: AnimatableTriggerEvent,
    #[serde(rename = "@action")] pub action: String,
}


#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all="camelCase")]
pub enum AnimatableTriggerEvent {
    Input,
    NoInput {
        #[serde(rename = "@duration")] duration: f32,
    },
    Hover,
    Unhover,
    Click,
    ClickHold {
        #[serde(rename = "@duration")] duration: f32,
    },
    Unclick,

    Event(TatakuEvent),
    Message(String)
}

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct AnimatableAction {
    #[serde(rename = "$value")] pub action: TransformTypeTag,
    #[serde(rename = "@duration")] pub duration: f32,
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum TransformTypeTag {
    #[default] None,
    Position {
        #[serde(alias="@start")] start: Vector2,
        #[serde(alias="@end")] end: Vector2,
    },
    PositionX {
        #[serde(rename="@start")] start: f32,
        #[serde(rename="@end")] end: f32
    },
    PositionY {
        #[serde(rename="@start")] start: f32,
        #[serde(rename="@end")] end: f32
    },
    VectorScale {
        #[serde(alias="@start")] start: Vector2,
        #[serde(alias="@end")] end: Vector2,
    },
    Scale {
        #[serde(rename="@start")] start: f32,
        #[serde(rename="@end")] end: f32
    },
    ScaleX {
        #[serde(rename="@start")] start: f32,
        #[serde(rename="@end")] end: f32
    },
    ScaleY {
        #[serde(rename="@start")] start: f32,
        #[serde(rename="@end")] end: f32
    },
    Rotation {
        #[serde(rename="@start")] start: f32,
        #[serde(rename="@end")] end: f32
    },
    // Color {
    //     #[serde(rename="@start")] start: Color,
    //     #[serde(rename="@end")] end: Color
    // },
    // BorderSize {
    //     #[serde(rename="@start")] start: f32,
    //     #[serde(rename="@end")] end: f32
    // },
    // Transparency {
    //     #[serde(rename="@start")] start: f32,
    //     #[serde(rename="@end")] end: f32
    // },
    // BorderTransparency {
    //     #[serde(rename="@start")] start: f32,
    //     #[serde(rename="@end")] end: f32
    // },
}
