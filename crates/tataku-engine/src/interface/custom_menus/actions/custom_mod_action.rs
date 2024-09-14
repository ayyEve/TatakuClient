use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };


/// An action that deals with the Song
#[derive(Clone, Debug)]
pub enum CustomModAction {
    /// Play/resume the song
    AddMod(CustomEventValueType),

    /// Pause the song
    RemoveMod(CustomEventValueType),

    /// Toggle the song (pause if playing, play if paused)
    ToggleMod(CustomEventValueType),

    /// Seek by the specified number of ms
    SetSpeed(CustomEventValueType),

    /// Set the song's position
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
impl<'lua> FromLua<'lua> for CustomModAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomModAction");
        match lua_value {
            Value::Table(table) => {

                macro_rules! check {
                    ($i: expr, $e: ident) => {
                        if let Some(n) = table.get($i)? {
                            return Ok(Self::$e(n))
                        }
                    }
                }

                check!("add", AddMod);
                check!("remove", RemoveMod);
                check!("toggle", ToggleMod);

                check!("set_speed", SetSpeed);
                check!("add_speed", AddSpeed);

                Err(FromLuaConversionError { 
                    from: "table", 
                    to: "CustomModAction", 
                    message: Some("couldn't determine mod action".to_string()) 
                })
            }


            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomModAction", message: None })
        }
    
    }
}
