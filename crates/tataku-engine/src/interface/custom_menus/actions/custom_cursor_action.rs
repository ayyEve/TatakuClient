use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub enum CustomMenuCursorAction {
    Show,
    Hide,
}
impl CustomMenuCursorAction {
    pub fn into_action(self, _values: &mut dyn Reflect, _passed_in: Option<TatakuValue>) -> Option<CursorAction> {
        match self {
            Self::Show => Some(CursorAction::SetVisible(true)),
            Self::Hide => Some(CursorAction::SetVisible(false)),
        }
    }
    
    pub fn build(&mut self, _values: &dyn Reflect) {}
}
impl<'lua> FromLua<'lua> for CustomMenuCursorAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        const THIS_TYPE: &str = "CustomMenuCursorAction"; 

        #[cfg(feature="debug_custom_menus")] info!("Reading {THIS_TYPE}");
        match lua_value {
            Value::String(str) => {
                #[cfg(feature="debug_custom_menus")] info!("Is String");
                match str.to_str()? {
                    "show" => Ok(Self::Show),
                    "hide" => Ok(Self::Hide),

                    other => Err(FromLuaConversionError { from: "String", to: THIS_TYPE, message: Some(format!("Invalid {THIS_TYPE} action: {other}")) }),
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: THIS_TYPE, message: None })
        }
    
    }
}
