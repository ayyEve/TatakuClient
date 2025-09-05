use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableAction {
    /// No action
    #[default] None,

    // A delayed action
    Delayed {
        #[serde(rename="$value")] action: Box<BuildableAction>,
        #[serde(rename="@delay")] delay: u64,
    },

    // An internal action at <path>
    Internal {
        #[serde(rename="$value")] path: VariablePathResolver,
    },

    /// Set a value
    SetValue {
        #[serde(rename="@key")] key: VariablePathResolver,
        #[serde(rename="$value", default = "empty_string")] value: BuildableValue,
    },

    /// Set the menu
    SetMenu {
        #[serde(rename="$value", default)]
        id: BuildableValue,
    },

    /// Add a dialog
    AddDialog {
        #[serde(rename="$value", alias="@id", default)]
        id: BuildableValue,

        #[serde(alias="@allow_multiple", default="_true")]
        allow_multiple: bool,

        #[serde(alias="@resizable", default)]
        resizable: bool,

        #[serde(alias="@draggable", default)]
        draggable: bool,

        #[serde(alias="@title", default)]
        title: String,
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

    /// Perform a ui action
    Ui {
        #[serde(rename="$value")]
        action: BuildableUiAction,
    },

    /// Perform an online content action
    OnlineContent {
        #[serde(rename="$value")]
        action: Box<BuildableOnlineContentAction>,
    },


    /// A conditional
    #[serde(alias="if")]
    Conditional {
        /// The condition to evaluate
        #[serde(rename="@condition", alias="@cond")] cond: BuildableCondition,

        /// What to do if true
        #[serde(rename = "true")] if_true_wrapped: Option<Wrapped<Box<BuildableAction>>>,
        #[serde(rename = "$value")] if_true: Option<Box<BuildableAction>>,

        /// What to do if false
        #[serde(rename = "false", alias="else", default)] if_false: Option<Wrapped<Box<BuildableAction>>>,
    },

    /// run a custom event
    #[serde(rename="custom")]
    CustomEvent {
        #[serde(rename="$value", default)]
        event: BuildableText,
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

                resizable,
                draggable,
                allow_multiple,
                title,
            } => {
                let id = id
                    .resolve(values, passed_in)
                    .map(Cow::into_owned)
                    .and_then(|i| i.string_maybe().cloned())?;

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
                }))
            }

            #[cfg(feature="graphics")]
            Self::CloseDialog
                => Some(UiAction::new(node, DialogAction::Close).into()),

            #[cfg(feature="graphics")]
            Self::SetMenu {
                id,
            } => {
                let id = id
                    .resolve(values, passed_in)
                    .map(Cow::into_owned)
                    .and_then(|i| i.string_maybe().cloned())?;

                Some(TatakuAction::Menu(MenuAction::SetMenu {
                    id: id.into(),
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

            #[cfg(feature="graphics")]
            Self::Ui { action } => {
                Some(UiAction::new(
                    node,
                    action.into_action(node, values, passed_in)?
                ).into())
            }

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
            Self::CustomEvent { mut event } => {
                event.compute().ok()?;

                Some(GameAction::HandleEvent(
                    TatakuEvent::CustomEvent(event.to_string(values)),
                    None
                ).into())
            }


            Self::Conditional {
                cond,
                if_true,
                if_true_wrapped,
                if_false
            } => {
                let if_true = if_true_wrapped
                    .map(|i| i.inner)
                    .or(if_true)?;

                match cond.resolve(values) {
                    BuildableConditionResult::Failed => None,
                    BuildableConditionResult::Unbuilt(a)
                        => panic!("BuildableConditions should be built! '{a}'"),
                    BuildableConditionResult::True => if_true
                        .into_action(node, values, passed_in),
                    BuildableConditionResult::False => if_false
                        .and_then(|a|
                            a.inner.into_action(node, values, passed_in)
                        ),
                    BuildableConditionResult::Error(_) => None,
                }
            }

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
            } => {
                id.resolve_pre(values);
            }
            Self::AddDialog {
                id,
                ..
            } => {
                id.resolve_pre(values);
            }
            Self::Conditional {
                cond,
                if_true,
                if_true_wrapped,
                if_false
            } => {
                cond.build();

                if let Some(e) = if_true {
                    e.build(values);
                }
                if let Some(e) = if_true_wrapped {
                    e.inner.build(values);
                }
                if let Some(e) = if_false {
                    e.inner.build(values);
                }
            }

            Self::Delayed { action, .. }
                => action.build(values),

            Self::SetValue { value, .. }
                => value.resolve_pre(values),

            Self::Chat { action }
                => action.build(values),

            Self::Ui { action }
                => action.build(values),

            Self::CustomEvent { event } => if let Err(e) = event.compute() {
                error!("error building custom event tag: {e:?}");
            }

            Self::Internal { .. } => {},
            Self::None => {},
            Self::CloseDialog => {},
            Self::Gameplay { .. } => {},
            Self::OnlineContent { .. } => {},
        }
    }
}

pub fn _true() -> bool { true }
pub fn empty_string() -> BuildableValue { BuildableValue::Value(TatakuValue::String("".to_owned())) }
