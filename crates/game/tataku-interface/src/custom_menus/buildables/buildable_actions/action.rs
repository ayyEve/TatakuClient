use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableAction {
    /// No action
    #[default] None,

    // A delayed action
    Delayed {
        #[serde(rename="$value")] action: Box<Self>,
        #[serde(rename="@delay")] delay: u64,
    },

    // An internal action at <path>
    Internal {
        #[serde(rename="@path")] path: VariablePathResolver,
    },

    /// Set a value
    SetValue {
        #[serde(rename="@key")] key: VariablePathResolver, 
        #[serde(rename="$value")] value: BuildableValue,
    },

    /// Set the menu
    SetMenu { 
        #[serde(rename="$value", default)] 
        id: Option<BuildableValue>,
        
        #[serde(alias="@id", default)] 
        id_attribute: Option<String>,

        #[serde(default)]
        variables: DialogInputsTag,
    },

    /// Add a dialog
    AddDialog { 
        #[serde(alias="$value", default)] 
        id: Option<BuildableValue>,
        
        #[serde(alias="@id", default)] 
        id_attribute: Option<String>,

        #[serde(alias="@allow_multiple", default="_true")] 
        allow_multiple: bool,

        #[serde(alias="@resizable", default)]
        resizable: bool,

        #[serde(alias="@draggable", default)]
        draggable: bool,

        #[serde(alias="@title", default)]
        title: String,

        #[serde(default)]
        variables: DialogInputsTag,
    },

    CloseDialog,

    /// Perform a map action
    Map { 
        #[serde(rename="$value")] 
        action: BuildableMapAction, 
    },

    /// Perform a mods action
    #[serde(alias = "mod")]
    Mods {
        #[serde(rename="$value")] 
        action: BuildableModAction,
    },

    /// Perform a song action
    Song { 
        #[serde(rename="$value")] 
        action: BuildableSongAction,
    },

    /// Perform a game action
    Game { 
        #[serde(rename="$value")] 
        action: BuildableGameAction
    },

    /// Perform a gameplay action
    Gameplay { 
        #[serde(rename="$value")] 
        action: BuildableGameplayAction
    },

    /// Perform a multiplayer action
    Multiplayer {
        #[serde(rename="$value")] 
        action: BuildableMultiplayerAction
    },

    /// Perform a cursor action
    Cursor {
        #[serde(rename="$value")] 
        action: BuildableCursorAction
    },

    /// Perform a chat action
    Chat {
        #[serde(rename="$value")] 
        action: BuildableChatAction,
    },

    /// Perform an online content action
    OnlineContent {
        #[serde(rename="$value")] 
        action: Box<BuildableOnlineContentAction>,
    },


    /// A conditional
    Conditional {
        /// The condition to evaluate
        #[serde(rename="@condition", alias="@cond")] cond: BuildableCondition,
        
        /// What to do if true
        #[serde(rename = "true")] if_true_specified: Option<Box<BuildableActionTag>>,
        #[serde(rename = "$value")] if_true: Option<Box<BuildableAction>>,

        /// What to do if false
        #[serde(rename = "false", default)] if_false: Option<Box<BuildableActionTag>>,
    },

    /// Run multiple actions
    #[serde(alias="list")]
    Multiple {
        #[serde(rename="$value")] actions: Vec<Self>
    },

    /// run a custom event
    #[serde(rename="custom")]
    CustomEvent {
        #[serde(rename="$value", default)]
        event_tag: Option<BuildableText>,

        #[serde(rename="@event", default)]
        event_attribute: Option<String>,
    }
}

impl BuildableAction {
    pub fn into_action(
        self, 
        node: NodeId,
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<TatakuAction> {
        match self {
            Self::None => None,
            Self::Delayed {
                action,
                delay,
            } => {
                let passed_in = passed_in.cloned();
                Some(TatakuAction::Delayed(
                    DelayedActionType::Callback(Arc::new(
                        move |values| action
                            .clone()
                            .into_action(node, values, passed_in.as_ref())
                            .unwrap_or(TatakuAction::None)
                    )),
                    delay
                ))
            }

            Self::Internal { path } => {
                let path = path
                    .resolve_path(values)
                    .map_err(|e| error!("{e}"))
                    .ok()?;

                let action = values
                    .reflect_get::<BuildableSettingsAction>(&path)
                    .map_err(|e| error!("{e:?}"))
                    .ok()?;

                action.inner.build(node, passed_in, values)
            }

            #[cfg(feature="graphics")] 
            Self::AddDialog { 
                id, 
                id_attribute,

                resizable,
                draggable,
                allow_multiple,
                title,

                variables 
            } => {
                let id = id
                    .and_then(|i| i
                        .resolve(values, passed_in)
                        .map(Cow::into_owned)
                    )
                    .and_then(|i| i.string_maybe().cloned())
                    .or(id_attribute)
                    ?;
                    
                Some(TatakuAction::Menu(MenuAction::AddDialog {
                    id: id.into(),
                    options: Box::new(DialogCreateOptions {
                        allow_multiple,
                        draggable,
                        resizable,
                        title: Cow::Owned(title),
                        // TODO: not auto?
                        location: DialogLocation::Auto,
                        background: true,
                    }),
                    input: Box::new(variables.build(values, passed_in))
                }))
            }
            
            #[cfg(feature="graphics")] 
            Self::CloseDialog 
                => Some(UiAction::new(node, DialogAction::Close).into()),
            
            #[cfg(feature="graphics")] 
            Self::SetMenu { 
                id, 
                id_attribute,
                variables 
            } => {
                let id = id
                    .and_then(|i| i.resolve(values, passed_in)
                        .map(Cow::into_owned)
                    )
                    .and_then(|i| i.string_maybe().cloned())
                    .or(id_attribute)
                    ?;

                Some(TatakuAction::Menu(MenuAction::SetMenu { 
                    id: id.into(), 
                    input: Box::new(variables.build(values, passed_in))
                }))
            }

            Self::Map { action } => action
                .into_action(values, passed_in)
                .map(TatakuAction::Beatmap),

            Self::Mods { action } => action
                .into_action(values, passed_in)
                .map(TatakuAction::Mods),

            Self::Song { action } => action
                .into_action(values, passed_in)
                .map(TatakuAction::Song),

            Self::Game { action } => action
                .into_action(values, passed_in)
                .map(Box::new)
                .map(TatakuAction::Game),

            Self::Multiplayer { action } => action
                .into_action(values, passed_in)
                .map(TatakuAction::Multiplayer),

            #[cfg(feature="graphics")] 
            Self::Cursor { action } => action
                .into_action(values, passed_in)
                .map(TatakuAction::CursorAction),

            Self::Chat { action } 
                => action.into_action(values, passed_in),

            Self::Gameplay { 
                action 
            } => Some(TatakuAction::Game(Box::new(
                GameAction::CurrentGameAction(action.into_action())
            ))),

            Self::OnlineContent { action } 
                => action
                .into_action(values, passed_in)
                .map(TatakuAction::OnlineContent),


            Self::SetValue { key, value } => {
                let key = key
                    .resolve_path(values)
                    .ok()?;
                
                value
                    .resolve(values, passed_in)
                    .map(|value| 
                        GameAction::SetValue(key, value.into_owned()).into()
                    )
            }
            Self::CustomEvent { 
                event_tag, 
                event_attribute 
            } => {
                let event = event_tag.and_then(|mut e| { 
                    e.compute().ok()?; 
                    Some(e.to_string(values)) 
                }).or(event_attribute)?;

                Some(GameAction::HandleEvent(
                    TatakuEventType::CustomEvent(event), 
                    None
                ).into())
            }
        

            Self::Conditional { 
                cond, 
                if_true, 
                if_true_specified,
                if_false 
            } => {
                let if_true = if_true_specified
                    .map(|i| i.action)
                    .or(if_true.map(|i| *i))?;

                match cond.resolve(values) {
                    BuildableConditionResult::Failed => None,
                    BuildableConditionResult::Unbuilt(a) 
                        => panic!("BuildableConditions should be built! '{a}'"),
                    BuildableConditionResult::True => if_true
                        .into_action(node, values, passed_in),
                    BuildableConditionResult::False => if_false
                        .and_then(|a| 
                            a.action.into_action(node, values, passed_in)
                        ),
                    BuildableConditionResult::Error(_) => None,
                }
            }

            Self::Multiple { 
                actions
            } => Some(TatakuAction::Multiple(actions
                .into_iter()
                .filter_map(|e| 
                    e.into_action(node, values, passed_in)
                )
                .collect())
            ),

            #[cfg(not(feature="graphics"))] _ => None
        }
    }

    // build any values that need to be built on item creation (ie, for lists that have temporary variables)
    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::Map { action } 
                => action.build(values),

            Self::Mods { action } 
                => action.build(values),
            Self::Song { action } 
                => action.build(values),
            Self::Game { action } 
                => action.build(values),
            Self::Multiplayer { action } 
                => action.build(values),
            Self::Cursor { action } 
                => action.build(values),
            Self::SetMenu { 
                id, 
                variables, 
                ..
            } => {
                variables.resolve_pre(values);
                if let Some(id) = id {
                    id.resolve_pre(values);
                }
            }
            Self::AddDialog { 
                id, 
                variables, 
                .. 
            } => {
                variables.resolve_pre(values);
                if let Some(id) = id {
                    id.resolve_pre(values);
                }
            }
            Self::Conditional { 
                cond, 
                if_true, 
                if_true_specified, 
                if_false 
            } => {
                cond.build();

                if let Some(e) = if_true {
                    e.build(values);
                }
                if let Some(e) = if_true_specified {
                    e.action.build(values);
                }
                if let Some(e) = if_false {
                    e.action.build(values);
                }
            }
            Self::Multiple { actions } => {
                for i in actions {
                    i.build(values);
                }
            }


            Self::Delayed { action, .. } 
                => action.build(values),

            Self::SetValue { value, .. } 
                => value.resolve_pre(values),

            Self::Chat { action } 
                => action.build(values),

            Self::CustomEvent { 
                event_tag: Some(event_tag), 
                .. 
            } => if let Err(e) = event_tag.compute() {
                error!("error building custom event tag: {e:?}");
            }

            Self::Internal { .. } => {},
            Self::None => {},
            Self::CloseDialog => {},
            Self::Gameplay { .. } => {},
            Self::OnlineContent { .. } => {},
            Self::CustomEvent { event_tag: None, .. } => {}
        }
    }
}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildableActionTag {
    #[serde(rename="$text", alias="$value")] 
    pub action: BuildableAction,
}
crate::impl_tag!(BuildableActionTag, BuildableAction, action);




#[derive(Deserialize)]
#[derive(Clone, Debug, PartialEq)]
pub struct DialogInput {
    #[serde(rename="@name")] pub name: String,
    #[serde(rename="$value", default)] pub value: Option<BuildableValue>,
    #[serde(rename="value", default)] pub value_tag: Option<BuildableValueTag>,
}
impl DialogInput {
    pub fn get_value(&self) -> Option<&BuildableValue> {
        self.value
            .as_ref()
            .or(self.value_tag
                .as_ref()
                .map(|i| &i.value)
            )
    }
    pub fn get_value_mut(&mut self) -> Option<&mut BuildableValue> {
        self.value
            .as_mut()
            .or(self.value_tag
                .as_mut()
                .map(|i| &mut i.value)
            )
    }

}

#[derive(Deserialize)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogInputsTag {
    #[serde(rename="$text", alias="$value")] pub inputs: Vec<DialogInput>,
}
impl DialogInputsTag {
    fn resolve_pre(&mut self, values: &dyn Reflect) {
        for i in self.inputs.iter_mut() {
            let Some(value) = i.get_value_mut() 
            else { continue };

            value.resolve_pre(values);
        }
    }

    pub fn build(
        self, 
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> BuildableInputArguments {
        let mut inputs = BuildableInputArguments::default();
        for i in self.inputs {
            let Some(value) = i.get_value().cloned() else { 
                error!("variable does not have a value!: {}", i.name);
                continue;
            };

            let Some(value) = value.resolve(values, passed_in) 
            else {
                error!("variable could not be resolved!: {value:?}");
                continue;
            };

            inputs.insert(i.name, value.into_owned());
        }

        inputs
    }
}



pub fn _true() -> bool { true }
