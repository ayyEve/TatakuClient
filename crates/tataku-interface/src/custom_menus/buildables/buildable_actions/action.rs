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
        #[serde(rename="$value")] 
        value: BuildableValue
    },

    /// Add a dialog
    AddDialog { 
        #[serde(rename="$value")] 
        value: BuildableValue,
    },

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
    pub fn into_action(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<TatakuAction> {
        match self {
            Self::None => None,
            Self::AddDialog { value } => {
                let val = value.resolve(values, passed_in)?;
                let str = val.string_maybe()?;
                Some(TatakuAction::Menu(MenuAction::AddDialogCustom(str.clone(), true)))
            }
            Self::SetMenu { value } =>  {
                let val = value.resolve(values, passed_in)?;
                let str = val.string_maybe()?;
                Some(TatakuAction::Menu(MenuAction::set_menu(str.clone())))
            }

            Self::Map { action } => action.into_action(values, passed_in).map(TatakuAction::Beatmap),
            Self::Mods { action } => action.into_action(values).map(TatakuAction::Mods),
            Self::Song { action } => action.into_action(values).map(TatakuAction::Song),
            Self::Game { action } => action.into_action(values, passed_in).map(Box::new).map(TatakuAction::Game),
            Self::Multiplayer { action } => action.into_action(values, passed_in).map(TatakuAction::Multiplayer),
            Self::Cursor { action } => action.into_action(values, passed_in).map(TatakuAction::CursorAction),

            Self::Gameplay { action } => Some(TatakuAction::Game(Box::new(GameAction::CurrentGameAction(action.into_action())))),

            Self::SetValue { key, value} => value
                .resolve(values, passed_in)
                .map(|value| GameAction::SetValue(key, value).into()),
        
        
        
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
                    BuildableConditionResult::True => if_true.into_action(values, passed_in),
                    BuildableConditionResult::False => if_false.and_then(|a| a.action.into_action(values, passed_in)),
                    BuildableConditionResult::Error(_) => None,
                }
            }

            Self::Multiple { actions } => {
                Some(TatakuAction::Multiple(
                    actions.into_iter()
                        .filter_map(|e| e.into_action(values, passed_in.clone()))
                        .collect()
                ))
            }
        
        }
    }

    // build any values that need to be built on item creation (ie, for lists that have temporary variables)
    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::Map { action } => action.build(values),
            Self::Mods { action } => action.build(values),
            Self::Song { action } => action.build(values),
            Self::Game { action } => action.build(values),
            Self::Multiplayer { action } => action.build(values),
            Self::Cursor { action } => action.build(values),

            Self::SetMenu { value} => value.resolve_pre(values),
            Self::AddDialog { value } => value.resolve_pre(values),

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
