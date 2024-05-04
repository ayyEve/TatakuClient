use crate::prelude::*;
use rlua::{ Value, FromLua, Error::FromLuaConversionError };

#[derive(Clone, Debug)]
pub enum CustomMenuGameAction {
    /// Quit the game
    Quit,

    /// View a score by id
    ViewScore(CustomEventValueType)
}
impl CustomMenuGameAction {
    pub fn into_action(self, values: &mut ValueCollection, passed_in: Option<TatakuValue>) -> Option<GameAction> {
        match self {
            Self::Quit => Some(GameAction::Quit),

            Self::ViewScore(score) => {
                // let num = match score {
                //     CustomEventValueType::None => return None,
                //     CustomEventValueType::Value(v) => v.as_u64().ok()?
                //     CustomEventValueType::Variable(var) => {
                //         let num = values.get_raw(&var).ok()?.as_u64().ok()?;
                //     }
                // };

                Some(GameAction::ViewScoreId(score.resolve(values, passed_in)?.as_u64().ok()? as usize))
            }
        }
    }
    
    pub fn build(&mut self, values: &ValueCollection, passed_in: Option<TatakuValue>) {
        let thing = match self {
            Self::ViewScore(score) => score,
            Self::Quit => return,
        };

        let Some(resolved) = thing.resolve(values, passed_in) else {
            error!("Couldn't resolve: {self:?}");
            return;
        };

        *thing = CustomEventValueType::Value(resolved);
    }
}
impl<'lua> FromLua<'lua> for CustomMenuGameAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        const THIS_TYPE:&str = "CustomMenuGameAction";

        #[cfg(feature="debug_custom_menus")] info!("Reading {THIS_TYPE}");
        match lua_value {

            Value::String(str) => {
                #[cfg(feature="debug_custom_menus")] info!("Is String");
                match str.to_str()? {
                    "quit" => Ok(Self::Quit),

                    other => Err(FromLuaConversionError { 
                        from: "String", 
                        to: THIS_TYPE, 
                        message: Some(format!("Unknown {THIS_TYPE} action: {other}")) 
                    }),
                }
            }

            Value::Table(table) => {
                let id:String = table.get("id")?;

                match &*id {
                    "quit" => Ok(Self::Quit),
                    "view_score" => Ok(Self::ViewScore(table.get("score_id")?)),

                    other => Err(FromLuaConversionError { 
                        from: "Table", 
                        to: THIS_TYPE, 
                        message: Some(format!("Unknown {THIS_TYPE} action: {other}")) 
                    }),
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: THIS_TYPE, message: None })
        }
    }
}
