use crate::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub enum BuildableAction {
    /// No action
    #[default] None,

    /// set a value
    SetValue {
        #[serde(rename="@key")] key: String, 
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

        #[serde(alias="@allow_duplicates", default="_true")] 
        allow_duplicates: bool,

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

    #[serde(alias="list")]
    Multiple {
        #[serde(rename="$value")] actions: Vec<Self>
    },
}

impl BuildableAction {
    pub fn into_action(
        self, 
        node: NodeId,
        values: &mut dyn Reflect, 
        passed_in: &Option<TatakuValue>
    ) -> Option<TatakuAction> {
        match self {
            Self::None => None,
            Self::AddDialog { 
                id, 
                id_attribute,
                allow_duplicates, 
                variables 
            } => {
                let id = id
                    .and_then(|i| i.resolve(values, passed_in).map(Cow::into_owned))
                    .and_then(|i| i.string_maybe().cloned())
                    .or(id_attribute)
                    ?;
                    
                Some(TatakuAction::Menu(MenuAction::AddDialog {
                    id: id.into(),
                    allow_duplicates,
                    input: variables.build(values, passed_in)
                }))
            }
            Self::CloseDialog => Some(UiAction::new(node, DialogAction::Close).into()),

            Self::SetMenu { 
                id, 
                id_attribute,
                variables 
            } =>  {
                let id = id
                    .and_then(|i| i.resolve(values, passed_in).map(Cow::into_owned))
                    .and_then(|i| i.string_maybe().cloned())
                    .or(id_attribute)
                    ?;

                Some(TatakuAction::Menu(MenuAction::SetMenu { 
                    id: id.into(), 
                    input: variables.build(values, passed_in)
                }))
            }

            Self::Map { action } => action.into_action(values, passed_in).map(TatakuAction::Beatmap),
            Self::Mods { action } => action.into_action(values, passed_in).map(TatakuAction::Mods),
            Self::Song { action } => action.into_action(values, passed_in).map(TatakuAction::Song),
            Self::Game { action } => action.into_action(values, passed_in).map(Box::new).map(TatakuAction::Game),
            Self::Multiplayer { action } => action.into_action(values, passed_in).map(TatakuAction::Multiplayer),
            Self::Cursor { action } => action.into_action(values, passed_in).map(TatakuAction::CursorAction),
            Self::Chat { action } => action.into_action(values, passed_in),

            Self::Gameplay { action } => Some(TatakuAction::Game(Box::new(GameAction::CurrentGameAction(action.into_action())))),

            Self::SetValue { key, value} => value
                .resolve(values, passed_in)
                .map(|value| GameAction::SetValue(key, value.into_owned()).into()),
        
        
        
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
                    BuildableConditionResult::Unbuilt(_) => unreachable!("BuildableConditions should be built!"),
                    BuildableConditionResult::True => if_true.into_action(node, values, passed_in),
                    BuildableConditionResult::False => if_false.and_then(|a| a.action.into_action(node, values, passed_in)),
                    BuildableConditionResult::Error(_) => None,
                }
            }

            Self::Multiple { actions } => {
                Some(TatakuAction::Multiple(
                    actions.into_iter()
                        .filter_map(|e| e.into_action(node, values, passed_in))
                        .collect()
                ))
            }
        
        }
    }

    // build any values that need to be built on item creation (ie, for lists that have temporary variables)
    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::Map  { 
                action
            } => action.build(values),
            
            Self::Mods { 
                action
            } => action.build(values),
            
            Self::Song { 
                action
            } => action.build(values),
            
            Self::Game { 
                action
            } => action.build(values),
            
            Self::Multiplayer { 
                action
            } => action.build(values),
            Self::Cursor { 
                action
            } => action.build(values),

            Self::SetMenu { 
                id, 
                variables, 
                ..
            } => {
                variables.resolve_pre(values);
                if let Some(id) = id {
                    id.resolve_pre(values);
                }
            },

            Self::AddDialog { 
                id, 
                variables, 
                .. 
            } => {
                variables.resolve_pre(values);
                if let Some(id) = id {
                    id.resolve_pre(values);
                }
            },

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

            _ => {}
        }
    }

    pub fn resolve(
        &self, 
        owner: MessageOwner, 
        values: &mut dyn Reflect, 
        passed_in: Option<TatakuValue>
    ) -> Option<Message> {
        if let BuildableAction::None = &self { return None };

        let mut action = self.clone();
        action.build(values);

        let value = Arc::new((action, passed_in));
        let message = MessageValue::Custom(value);
        Some(Message::new(owner, "", message))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct BuildableActionTag {
    #[serde(rename="$text", alias="$value")] pub action: BuildableAction,
}
crate::impl_tag!(BuildableActionTag, BuildableAction, action);




#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq)]
#[derive(Deserialize)]
pub struct DialogInputsTag {
    #[serde(rename="$text", alias="$value")] pub inputs: Vec<DialogInput>,
}
impl DialogInputsTag {
    fn resolve_pre(&mut self, values: &dyn Reflect) {
        for i in self.inputs.iter_mut() {
            let Some(value) = i.get_value_mut() else { continue };
            value.resolve_pre(values);
        }
    }

    pub fn build(
        self, 
        values: &dyn Reflect,
        passed_in: &Option<TatakuValue>
    ) -> BuildableInputArguments {
        let mut inputs = BuildableInputArguments::default();
        for i in self.inputs {
            let Some(value) = i.get_value().cloned() else { 
                error!("variable does not have a value!: {}", i.name);
                continue;
            };
            let Some(value) = value.resolve(values, passed_in) else {
                error!("variable could not be resolved!: {value:?}");
                continue;
            };
            inputs.insert(i.name, value.into_owned());
        }

        inputs
    }
}



pub fn _true() -> bool { true }