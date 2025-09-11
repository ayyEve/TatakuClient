use crate::prelude::*;
use ui::tree::NodeId;
use tataku::TatakuValue;
use common::reflect::Reflect;
use engine::VariablePathResolver;
use engine::actions;
// use engine::actions::{ 
//     UiAction,
//     GameAction,
//     MenuAction,
//     actions::Action,
//     DialogLocation,
//     DialogCreateOptions,
//     VariablePathResolver, 
// };

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableAction {
    /// No action
    #[default] None,

    // A delayed action
    Delayed {
        #[serde(rename="$value")] actions: Vec<BuildableAction>,
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
        #[serde(rename = "$value")] if_true: Vec<BuildableAction>,

        /// What to do if false
        #[serde(rename = "false", alias="else", default)] if_false: Option<Wrapped<Vec<BuildableAction>>>,
    },

    /// run a custom event
    #[serde(rename="custom")]
    CustomEvent {
        #[serde(rename="$value", default)]
        event: BuildableValue,
    }
}
impl BuildableAction {
    pub fn resolve(
        &self,
        node: NodeId,
        values: &mut dyn Reflect,
        passed_in: Option<&TatakuValue>
    ) -> Option<actions::Action> {
        match self {
            Self::None => None,
            Self::Delayed {
                actions,
                delay,
            } => {
                let passed_in = passed_in.cloned();
                let actions = actions.clone();

                let delayed = actions::action::DelayedActionType::Callback(Arc::new(
                    move |values| actions::Action::Multiple(actions
                        .iter()
                        .filter_map(|a| a.resolve(node, values, passed_in.as_ref()))
                        .collect()
                )));

                Some(actions::Action::Delayed(
                    delayed,
                    *delay
                ))
            }

            Self::Internal { path } => {
                let path = path
                    .resolve_path(values)
                    .map_err(|e| error!("{e}"))
                    .ok()?;

                let action = values
                    .reflect_get::<engine::settings::BuildableSettingsAction>(&path)
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

                Some(actions::Action::Menu(engine::actions::menu::MenuAction::AddDialog {
                    id: id.into(),
                    options: Box::new(actions::menu::DialogCreateOptions {
                        allow_multiple: *allow_multiple,
                        draggable: *draggable,
                        resizable: *resizable,
                        title: Cow::Owned(title.clone()),
                        // TODO: not auto?
                        location: actions::menu::DialogLocation::Auto,
                        background: true,
                    }),
                }))
            }

            #[cfg(feature="graphics")]
            Self::CloseDialog
                => Some(actions::ui::UiAction::new(node, engine::actions::dialog::DialogAction::Close).into()),

            #[cfg(feature="graphics")]
            Self::SetMenu {
                id,
            } => {
                let id = id
                    .resolve(values, passed_in)
                    .map(Cow::into_owned)
                    .and_then(|i| i.string_maybe().cloned())?;

                Some(actions::Action::Menu(actions::menu::MenuAction::SetMenu {
                    id: id.into(),
                }))
            }

            Self::Map { action } => action
                .resolve(values, passed_in)
                .map(actions::Action::Beatmap),

            Self::Mods { action } => action
                .resolve(values, passed_in)
                .map(actions::Action::Mods),

            Self::Song { action } => action
                .resolve(values, passed_in)
                .map(actions::Action::Song),

            Self::Game { action } => action
                .resolve(values, passed_in)
                .map(Box::new)
                .map(actions::Action::Game),

            Self::Multiplayer { action } => action
                .resolve(values, passed_in)
                .map(actions::Action::Multiplayer),

            #[cfg(feature="graphics")]
            Self::Cursor { action } => action
                .resolve(values, passed_in)
                .map(actions::Action::CursorAction),

            Self::Chat { action }
                => action.resolve(values, passed_in),

            #[cfg(feature="graphics")]
            Self::Ui { action } => {
                Some(actions::ui::UiAction::new(
                    node,
                    action.resolve(node, values, passed_in)?
                ).into())
            }

            Self::Gameplay {
                action
            } => Some(actions::Action::Game(Box::new(
                actions::game::GameAction::CurrentGameAction(action.resolve())
            ))),

            Self::OnlineContent { action }
                => action
                .resolve(values, passed_in)
                .map(actions::Action::OnlineContent),


            Self::SetValue { key, value } => {
                let key = key
                    .resolve_path(values)
                    .ok()?;

                value
                    .resolve(values, passed_in)
                    .map(|value|
                        actions::game::GameAction::SetValue(key, value.into_owned()).into()
                    )
            }
            Self::CustomEvent { event } => {
                let event = event.resolve(values , passed_in)?;
                let event = event.as_string();

                Some(actions::game::GameAction::HandleEvent(
                    input::TatakuEvent::CustomEvent(event.into()),
                    None
                ).into())
            }


            Self::Conditional {
                cond,
                if_true,
                if_false
            } => {
                match cond.resolve(values) {
                    BuildableConditionResult::Failed => None,
                    BuildableConditionResult::Unbuilt(a)
                        => panic!("BuildableConditions should be built! '{a}'"),
                    BuildableConditionResult::True => Some(actions::Action::Multiple(
                        if_true.iter()
                        .filter_map(|a| a.resolve(node, values, passed_in))
                        .collect()
                    )),
                    BuildableConditionResult::False => if_false.as_ref()
                        .map(|if_false| actions::Action::Multiple(
                            if_false.inner.iter()
                                .filter_map(|a| a.resolve(node, values, passed_in))
                                .collect()
                        )),
                    BuildableConditionResult::Error(_) => None,
                }
            }

            #[cfg(not(feature="graphics"))] _ => None
        }
    }

    // build any values that need to be built on item creation (ie, for lists that have temporary variables)
    pub fn build(&mut self) {
        match self {
            Self::Map { action }
                => action.build(),

            Self::Mods { action }
                => action.build(),
            Self::Song { action }
                => action.build(),
            Self::Game { action }
                => action.build(),
            Self::Multiplayer { action }
                => action.build(),
            Self::Cursor { action }
                => action.build(),
            Self::SetMenu { id } 
                => id.build(),
            
            Self::AddDialog { id, .. } 
            => id.build(),
            
            Self::Conditional {
                cond,
                if_true,
                if_false
            } => {
                cond.build();

                for action in if_true {
                    action.build();
                }

                for action in if_false.iter_mut().flat_map(|a| &mut a.inner) {
                    action.build();
                }
            }

            Self::Delayed { actions, .. } => {
                for action in actions {
                    action.build();
                }
            },

            Self::SetValue { value, .. }
                => value.build(),

            Self::Chat { action }
                => action.build(),

            Self::Ui { action }
                => action.build(),

            Self::CustomEvent { event }
                => event.build(),

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
