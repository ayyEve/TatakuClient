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

    /// Perform a mods action
    Mods(CustomModAction),

    /// Perform a song action
    Song(CustomMenuSongAction),

    /// Perform a game action
    Game(CustomMenuGameAction),

    /// Perform a multiplayer action
    Multiplayer(CustomMenuMultiplayerAction),

    /// set a value
    SetValue(String, TatakuValue),
}
impl CustomMenuAction {
    pub fn into_action<'a>(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<TatakuAction> {
        match self {
            Self::None => None,
            Self::AddDialog(dialog) => {
                let Some(val) = dialog.resolve(values, passed_in) else { return None };
                let str = val.string_maybe()?;
                Some(TatakuAction::Menu(MenuAction::AddDialogCustom(str.clone(), true)))
            }
            Self::SetMenu(menu) =>  {
                let Some(val) = menu.resolve(values, passed_in) else { return None };
                let str = val.string_maybe()?;
                Some(TatakuAction::Menu(MenuAction::set_menu(str.clone())))
            }

            Self::Map(action) => action.into_action(values, passed_in).map(TatakuAction::Beatmap),
            Self::Mods(action) => action.into_action(values).map(TatakuAction::Mods),
            
            Self::Song(action) => action.into_action(values).map(TatakuAction::Song),
            Self::Game(action) => action.into_action(values, passed_in).map(TatakuAction::Game),
            Self::Multiplayer(action) => action.into_action(values).map(TatakuAction::Multiplayer),
            
            Self::SetValue(key, val) => Some(TatakuAction::Game(GameAction::SetValue(key, val))),
        }
    }

    // build any values that need to be built on item creation (ie, for lists that have temporary variables)
    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::Map(action) => action.build(values),
            Self::Mods(action) => action.build(values),
            Self::Song(action) => action.build(values),
            Self::Game(action) => action.build(values),
            Self::Multiplayer(action) => action.build(values),


            Self::SetMenu(menu) => {
                menu.resolve_pre(values);
                // if let Some(val) = menu.resolve_pre(values) {
                //     *menu = CustomEventValueType::Value(val);
                // } else {
                //     error!("failed to resolve menu from variable: {menu:?}")
                // }
            }
            Self::AddDialog(dialog) => {
                dialog.resolve_pre(values);
                // if let Some(val) = dialog.resolve_pre(values) {
                //     *dialog = CustomEventValueType::Value(val);
                // } else {
                //     error!("failed to resolve dialog from variable: {dialog:?}")
                // }
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
        // mod actions
        else if let Some(mod_action) = table.get::<_, Option<_>>("mods")? {
            Ok(Self::Mods(mod_action))
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
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuAction");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "CustomMenuAction", message: Some("Not a table".to_owned()) }) };
        Self::from_table(&table)
    }
}
