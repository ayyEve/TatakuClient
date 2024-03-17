use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError, Table };

#[derive(Clone, Debug)]
pub enum CustomMenuAction {
    /// No action
    None,

    /// Set the menu
    SetMenu(CustomEventValueType),

    /// Add a dialog
    AddDialog(CustomEventValueType),

    /// Perform a map action
    Map(CustomMenuMapAction),

    /// Perform a song action
    Song(CustomMenuSongAction),

    /// Perform a game action
    Game(CustomMenuGameAction),

    /// Perform a multiplayer action
    Multiplayer(CustomMenuMultiplayerAction),

    /// run a custom action
    CustomEvent(String, String),

    /// set a value
    SetValue(String, CustomElementValue)
}
impl CustomMenuAction {
    pub fn into_action(self, values: &mut ValueCollection) -> TatakuAction {
        match self {
            Self::None => TatakuAction::None,
            Self::AddDialog(dialog) => {
                let Some(val) = dialog.resolve(values) else { return TatakuAction::None };
                TatakuAction::Menu(MenuMenuAction::AddDialogCustom(val.as_string(), true))
            }
            Self::SetMenu(menu) =>  {
                let Some(val) = menu.resolve(values) else { return TatakuAction::None };
                TatakuAction::Menu(MenuMenuAction::SetMenu(val.as_string()))
            }

            Self::Map(action) => TatakuAction::Beatmap(action.into_action(values)),
            Self::Song(action) => TatakuAction::Song(action.into_action(values)),
            Self::Game(action) => TatakuAction::Game(action.into_action(values)),
            Self::Multiplayer(action) => {
                if let Some(action) = action.into_action(values) {
                    TatakuAction::Multiplayer(action)
                } else {
                    TatakuAction::None
                }
            }
            
            Self::CustomEvent(_, _) => unimplemented!(),
            Self::SetValue(key, val) => TatakuAction::Game(GameAction::SetValue(key, val)),
        }
    }

    // build any values that need to be built on item creation (ie, for lists that have temporary variables)
    pub fn resolve(&mut self, values: &ValueCollection) {
        match self {
            Self::Multiplayer(action) => action.resolve(values),
            Self::SetMenu(menu) => {
                if let Some(val) = menu.resolve(values) {
                    *menu = CustomEventValueType::Value(val);
                } else {
                    error!("failed to resolve menu from variable: {menu:?}")
                }
            }
            Self::AddDialog(dialog) => {
                if let Some(val) = dialog.resolve(values) {
                    *dialog = CustomEventValueType::Value(val);
                } else {
                    error!("failed to resolve dialog from variable: {dialog:?}")
                }
            }

            _ => {}
        }
    }

    /// parse this directly from a table (mainly used by ButtonAction)
    pub fn from_table(table: &Table) -> rlua::Result<Self> {
        // menu actions
        if let Some(action_str) = table.get::<_, Option<CustomEventValueType>>("menu")? {
            Ok(Self::SetMenu(action_str))
        }
        // dialog actions
        else if let Some(action_str) = table.get::<_, Option<CustomEventValueType>>("dialog")? {
            Ok(Self::AddDialog(action_str))
        }
        // beatmap actions
        else if let Some(map_action) = table.get::<_, Option<CustomMenuMapAction>>("map")? {
            Ok(Self::Map(map_action))
        }
        // song actions
        else if let Some(song_action) = table.get::<_, Option<CustomMenuSongAction>>("song")? {
            Ok(Self::Song(song_action))
        }
        // multiplayer actions
        else if let Some(multiplayer_action) = table.get::<_, Option<CustomMenuMultiplayerAction>>("multiplayer")? {
            Ok(Self::Multiplayer(multiplayer_action))
        }
        // game actions
        else if let Some(game_action) = table.get::<_, Option<CustomMenuGameAction>>("game")? {
            Ok(Self::Game(game_action))
        }

        // nope
        else {
            Err(FromLuaConversionError { 
                from: "Table", 
                to: "CustomMenuAction", 
                message: Some(format!("Could not determine action from table: {table:?}")) 
            })
        }
    }
}
impl<'lua> FromLua<'lua> for CustomMenuAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading CustomMenuAction");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "CustomMenuAction", message: Some("Not a table".to_owned()) }) };
        Self::from_table(&table)
    }
}
