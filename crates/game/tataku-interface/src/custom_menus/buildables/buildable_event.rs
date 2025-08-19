use crate::prelude::*;

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct BuildableEvent {
    #[serde(default)] pub event_tag: Option<BuildableEventTypeTag>,
    #[serde(rename="$value", default)] pub event: Option<BuildableEventType>,
    pub actions: BuildableActionsTag,
}
impl BuildableEvent {
    pub fn get_event(&self) -> Option<&BuildableEventType> {
        self.event.as_ref()
            .or(self.event_tag.as_ref().map(|i| &i.event))
    }

    pub fn get_actions(&self) -> Vec<BuildableAction> {
        self.actions.iter()
            .cloned()
            .map(|mut value| {
                if let BuildableAction::Conditional { cond, .. } = &mut value {
                    cond.build();
                }
                value
            })
            .collect()
    }
}


#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableEventType {
    /// Song has started
    SongStart,

    /// Song was paused
    SongPause,

    /// Song has ended
    SongEnd,

    /// Menu was entered
    MenuEnter,
    
    // Menu was left
    MenuLeave,

    /// A new beatmap has been added
    MapAdded,

    /// A key press
    KeyPress(CustomMenuKeyEvent),

    /// A key release
    KeyRelease(CustomMenuKeyEvent),

    /// A controller button was pressed
    ControllerPress(CustomMenuControllerEvent),

    /// A controller button was released
    ControllerRelease(CustomMenuControllerEvent),

    /// A custom event
    #[serde(rename="custom")]
    CustomEvent {
        #[serde(rename="$value", default)]
        event_tag: Option<BuildableText>,

        #[serde(rename="@event", default)]
        event_attribute: Option<String>,
    }
}
impl BuildableEventType {
    pub fn build(&mut self) {
        #[allow(clippy::single_match, reason = "expandability")]
        match self {
            Self::CustomEvent {
                event_tag: Some(e),
                ..
            } => e
                .compute()
                .inspect_err(|e| error!("error with text: {e:?}"))
                .nope(),

            _ => {}
        }
    }

    pub fn resolve(
        &self, 
        values: &dyn Reflect, 
        // passed_in: Option<&TatakuValue>,
    ) -> Option<TatakuEventType> {
        match self {
            Self::SongStart => Some(TatakuEventType::SongStart),
            Self::SongPause => Some(TatakuEventType::SongPause),
            Self::SongEnd   => Some(TatakuEventType::SongEnd),

            Self::MenuEnter => Some(TatakuEventType::MenuEnter),
            Self::MenuLeave => Some(TatakuEventType::MenuLeave),

            Self::MapAdded => Some(TatakuEventType::MapAdded),

            Self::KeyPress(k)   => Some(TatakuEventType::KeyPress(*k)),
            Self::KeyRelease(k) => Some(TatakuEventType::KeyRelease(*k)),

            Self::ControllerPress(k) => Some(TatakuEventType::ControllerPress(*k)),
            Self::ControllerRelease(k) => Some(TatakuEventType::ControllerRelease(*k)),

            Self::CustomEvent {
                event_tag,
                event_attribute,
            } => event_tag
                .as_ref()
                .map(|i| i.to_string(values))
                .or(event_attribute.clone())
                .map(TatakuEventType::CustomEvent),
        }

    }
}



#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct BuildableEventTypeTag {
    #[serde(rename="$value")] pub event: BuildableEventType,
}
impl Deref for BuildableEventTypeTag {
    type Target = BuildableEventType;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct BuildableActionsTag {
    #[serde(rename="$value")] pub actions: Vec<BuildableAction>
}
impl Deref for BuildableActionsTag {
    type Target = Vec<BuildableAction>;
    fn deref(&self) -> &Self::Target { &self.actions }
}


#[test]
fn test() {
    quick_xml::de::from_str::<BuildableEvent>(r#"
        <event>
            <event><songEnd/></event>
            <actions>
                <map><next/></map>
            </actions>
        </event>
    "#)
    .map_err(|e| TatakuError::String(format!("{e}")))
    .unwrap();
}
