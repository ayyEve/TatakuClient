use crate::prelude::*;
use rlua::{
    Value, 
    Error::FromLuaConversionError, 
    FromLua,
    Result,
    prelude::LuaContext,
};


#[derive(Clone, Debug)]
pub enum ComponentDef {
    // /// Provides a list of scores 
    // ScoreList,

    /// Provides a list of lobbies 
    LobbyList,
}
impl ComponentDef {
    pub async fn build(&self) -> Box<dyn Widgetable> {
        match self {
            // Self::ScoreList => Box::new(ScoreListComponent::new()),
            Self::LobbyList => Box::new(LobbyListComponent::default()),
        }
    }
}


impl<'lua> FromLua<'lua> for ComponentDef {
    fn from_lua(lua_value: Value<'lua>, _lua: LuaContext<'lua>) -> Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading ComponentDef");
        match lua_value {
            Value::Table(table) => {
                
                let id:String = table.get("id")?;
                #[cfg(feature="debug_custom_menus")] info!("Is table");
                match &*id {
                    
                    // "score_list" => Ok(Self::ScoreList),
                    "lobby_list" => Ok(Self::LobbyList),

                    _ => Err(FromLuaConversionError { from: "Table", to: "ComponentDef", message: Some("Could not determine type".to_owned()) })
                }
            }

            Value::String(s) => {
                #[cfg(feature="debug_custom_menus")] info!("Is string");
                match s.to_str().unwrap() {
                    // "beatmap_list" => Ok(Self::BeatmapList { filter_var: None }),
                    // "score_list" => Ok(Self::ScoreList),
                    "lobby_list" => Ok(Self::LobbyList),

                    _ => Err(FromLuaConversionError { from: "String", to: "ComponentDef", message: Some("Could not determine type".to_owned()) })
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "ComponentDef", message: Some("Incorrect type".to_owned()) })
        }
    }
}
