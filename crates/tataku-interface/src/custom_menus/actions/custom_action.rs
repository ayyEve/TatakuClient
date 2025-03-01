use crate::prelude::*;
use lua::*;

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

    /// Perform a cursor action
    Cursor(CustomMenuCursorAction),

    /// set a value
    SetValue(String, CustomEventValueType),
}
impl CustomMenuAction {
    pub fn into_action(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<TatakuAction> {
        match self {
            Self::None => None,
            Self::AddDialog(dialog) => {
                let val = dialog.resolve(values, passed_in)?;
                let str = val.string_maybe()?;
                Some(TatakuAction::Menu(MenuAction::AddDialogCustom(str.clone(), true)))
            }
            Self::SetMenu(menu) =>  {
                let val = menu.resolve(values, passed_in)?;
                let str = val.string_maybe()?;
                Some(TatakuAction::Menu(MenuAction::set_menu(str.clone())))
            }

            Self::Map(action) => action.into_action(values, passed_in).map(TatakuAction::Beatmap),
            Self::Mods(action) => action.into_action(values).map(TatakuAction::Mods),
            
            Self::Song(action) => action.into_action(values).map(TatakuAction::Song),
            Self::Game(action) => action.into_action(values, passed_in).map(Box::new).map(TatakuAction::Game),
            Self::Multiplayer(action) => action.into_action(values).map(TatakuAction::Multiplayer),
            Self::Cursor(action) => action.into_action(values, passed_in).map(TatakuAction::CursorAction),

            Self::SetValue(key, val) => val
                .resolve(values, passed_in)
                .map(|value| GameAction::SetValue(key, value).into()),
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
            Self::Cursor(action) => action.build(values),


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
    pub fn from_table(table: &LuaTable) -> LuaResult<Self> {
        // menu actions
        if let Some(action_str) = table.get::<Option<CustomEventValueType>>("menu")? {
            Ok(Self::SetMenu(action_str))
        }
        // dialog actions
        else if let Some(action_str) = table.get::<Option<CustomEventValueType>>("dialog")? {
            Ok(Self::AddDialog(action_str))
        }
        // beatmap actions
        else if let Some(action) = table.get::<Option<CustomMenuMapAction>>("map")? {
            Ok(Self::Map(action))
        }
        // mod actions
        else if let Some(action) = table.get::<Option<_>>("mods")? {
            Ok(Self::Mods(action))
        }
        // song actions
        else if let Some(action) = table.get::<Option<CustomMenuSongAction>>("song")? {
            Ok(Self::Song(action))
        }
        // multiplayer actions
        else if let Some(action) = table.get::<Option<CustomMenuMultiplayerAction>>("multiplayer")? {
            Ok(Self::Multiplayer(action))
        }
        // game actions
        else if let Some(action) = table.get::<Option<CustomMenuGameAction>>("game")? {
            Ok(Self::Game(action))
        }
        // cursor actions
        else if let Some(action) = table.get::<Option<CustomMenuCursorAction>>("cursor")? {
            Ok(Self::Cursor(action))
        }

        // nope
        else {
            Err(FromLuaConversionError { 
                from: "Table", 
                to: "CustomMenuAction".to_owned(),
                message: Some(format!("Could not determine action from table: {table:?}")) 
            })
        }
    }
}
impl FromLua for CustomMenuAction {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomMenuAction");
        let LuaValue::Table(table) = lua_value else { return Err(FromLuaConversionError { 
            from: lua_value.type_name(), 
            to: "CustomMenuAction".to_owned(), 
            message: Some("Not a table".to_owned()) 
        }) };
        Self::from_table(&table)
    }
}
