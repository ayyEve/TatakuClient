use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub enum CustomMenuGameAction {
    /// Quit the game
    Quit,

    /// View a score
    ViewScore(IngameScore),
}
impl CustomMenuGameAction {
    pub fn into_action(self, _values: &mut ValueCollection) -> GameAction {
        match self {
            Self::Quit => GameAction::Quit,
            Self::ViewScore(score) => GameAction::ViewScore(score),
        }
    }
}
impl<'lua> FromLua<'lua> for CustomMenuGameAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading CustomMenuGameAction");
        match lua_value {

            Value::String(str) => {
                #[cfg(feature="custom_menu_debugging")] info!("Is String");
                match str.to_str()? {
                    "quit" => Ok(Self::Quit),

                    other => Err(FromLuaConversionError { 
                        from: "String", 
                        to: "CustomMenuGameAction", 
                        message: Some(format!("Unknown action: {other}")) 
                    }),
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomMenuSongAction", message: None })
        }
    
    }
}
