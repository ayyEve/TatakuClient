use crate::prelude::*;
use lua::*;

/// An action that deals with the Mod manager
#[derive(Clone, Debug)]
pub enum CustomModAction {
    /// Add the specified mod
    AddMod(CustomEventValueType),

    /// Remove the specified mod
    RemoveMod(CustomEventValueType),

    /// Toggle the specified mod
    ToggleMod(CustomEventValueType),

    /// Set the gameplay speed to the specified value
    SetSpeed(CustomEventValueType),

    /// add/subtract to/from the gameplay speed by the specified amount
    AddSpeed(CustomEventValueType),
}
impl CustomModAction {
    pub fn into_action(self, values: &mut dyn Reflect) -> Option<ModAction> {
        match self {
            Self::AddMod(n) => n.resolve(values, None).and_then(|n| n.string_maybe().cloned()).map(ModAction::AddMod),
            Self::RemoveMod(n) => n.resolve(values, None).and_then(|n| n.string_maybe().cloned()).map(ModAction::RemoveMod),
            Self::ToggleMod(n) => n.resolve(values, None).and_then(|n| n.string_maybe().cloned()).map(ModAction::ToggleMod),
            Self::SetSpeed(n) => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(ModAction::SetSpeed),
            Self::AddSpeed(n) => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(ModAction::AddSpeed),
        }
    }

    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::AddMod(n) => n.resolve_pre(values),
            Self::RemoveMod(n) => n.resolve_pre(values),
            Self::ToggleMod(n) => n.resolve_pre(values),
            Self::SetSpeed(n) => n.resolve_pre(values),
            Self::AddSpeed(n) => n.resolve_pre(values),
        }
    }
}
impl FromLua for CustomModAction {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomModAction");
        match lua_value {
            LuaValue::Table(table) => {
                type F = fn(CustomEventValueType) -> CustomModAction;
                for (i, e) in [
                    ("add", Self::AddMod as F),
                    ("remove", Self::RemoveMod as F),
                    ("toggle", Self::ToggleMod as F),

                    ("set_speed", Self::SetSpeed as F),
                    ("add_speed", Self::AddSpeed as F),
                ] {
                    let Some(n) = table.get(i)? else { continue };
                    return Ok(e(n))
                }
                Err(FromLuaConversionError { 
                    from: "table", 
                    to: "CustomModAction".to_owned(), 
                    message: Some("couldn't determine mod action".to_string()) 
                })
            }

            other => Err(FromLuaConversionError { 
                from: other.type_name(), 
                to: "CustomModAction".to_owned(), 
                message: None
            })
        }
    
    }
}
