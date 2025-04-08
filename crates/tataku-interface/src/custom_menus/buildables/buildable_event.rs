use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableEvent {
    #[serde(default)] pub event_explicit: Option<TatakuEventTypeTag>,
    #[serde(rename="$value", default)] pub event_inner: Option<TatakuEventType>,
    pub actions: BuildableActionsTag,
}
impl BuildableEvent {
    pub fn get_event(&self) -> Option<&TatakuEventType> {
        self.event_inner.as_ref()
            .or(self.event_explicit.as_ref().map(|i| &i.event))
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


#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct TatakuEventTypeTag {
    #[serde(rename="$value", alias="$text")] pub event: TatakuEventType,
}
impl Deref for TatakuEventTypeTag {
    type Target = TatakuEventType;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableActionsTag {
    #[serde(rename="$value", alias="$text")] pub actions: Vec<BuildableAction>
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
