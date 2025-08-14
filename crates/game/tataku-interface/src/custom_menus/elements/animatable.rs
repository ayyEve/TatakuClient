use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct AnimatableElement {
    #[serde(rename = "@id", default)] id: Option<ArcStr>,
    #[serde(rename = "@class", default)] class_list: ClassList,
    #[serde(rename = "@style", default)] style: ArcStr,
    
    #[serde(default)] triggers: AnimatableTriggersTag,
    #[serde(default)] actions: AnimatableActionsTag,

    element: ElementTag,
}
impl CustomElement for AnimatableElement {
    fn build(&self) -> Box<dyn Widget<TatakuAction>> {
        WidgetContainer::new_boxed(
            self.style.clone(),
            "animatable",
            self.id.clone(),
            self.class_list.clone(),
            TransformableWidget::new(
                self.triggers.triggers.clone(),
                self.actions.iter().cloned().map(|i| (i.id, i.list)).collect(),
                self.element.build()
            )
            .boxed()
        )
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
struct AnimatableActionEntry {
    #[serde(rename = "@id")] id: String,
    #[serde(alias = "$value")] list: Vec<AnimatableAction>
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
struct AnimatableActionsTag {
    #[serde(alias = "$value")] entries: Vec<AnimatableActionEntry>
}
crate::impl_tag!(AnimatableActionsTag, Vec<AnimatableActionEntry>, entries);



#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct AnimatableTrigger {
    #[serde(alias = "$value")] pub trigger: AnimatableTriggerEvent,
    #[serde(rename = "@action")] pub action: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct AnimatableTriggersTag {
    #[serde(alias = "$value")] pub triggers: Vec<AnimatableTrigger>,
}
crate::impl_tag!(AnimatableTriggersTag, Vec<AnimatableTrigger>, triggers);


#[derive(Clone, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
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

    Event(TatakuEventType),
    Message(String)
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct AnimatableAction {
    #[serde(rename = "$value")] pub action: TransformTypeTag,
    #[serde(rename = "@duration")] pub duration: f32,
}


#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
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
