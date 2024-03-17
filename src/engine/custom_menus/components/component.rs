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
    /// Provides beatmap list information
    BeatmapList { 
        /// What variable to use for the beatmap filter
        filter_var: Option<String>, 
    },
    /// Provides a list of scores 
    ScoreList,

    /// Provides a list of lobbies 
    LobbyList,
}
impl ComponentDef {
    pub async fn build(&self) -> Box<dyn Widgetable> {
        match self {
            Self::BeatmapList { filter_var } => Box::new(BeatmapListComponent::new(filter_var.clone())),
            Self::ScoreList => Box::new(ScoreListComponent::new()),
            Self::LobbyList => Box::new(LobbyListComponent::default()),
        }
    }
}


impl<'lua> FromLua<'lua> for ComponentDef {
    fn from_lua(lua_value: Value<'lua>, _lua: LuaContext<'lua>) -> Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading ComponentDef");
        match lua_value {
            Value::Table(table) => {
                
                let id:String = table.get("id")?;
                #[cfg(feature="custom_menu_debugging")] info!("Is table");
                match &*id {
                    "beatmap_list" => Ok(Self::BeatmapList {
                        filter_var: table.get("filter_var")?,
                    }),
                    
                    "score_list" => Ok(Self::ScoreList),
                    "lobby_list" => Ok(Self::LobbyList),

                    _ => Err(FromLuaConversionError { from: "Table", to: "ComponentDef", message: Some("Could not determine type".to_owned()) })
                }
            }

            Value::String(s) => {
                #[cfg(feature="custom_menu_debugging")] info!("Is string");
                match s.to_str().unwrap() {
                    "beatmap_list" => Ok(Self::BeatmapList { filter_var: None }),
                    "score_list" => Ok(Self::ScoreList),
                    "lobby_list" => Ok(Self::LobbyList),

                    _ => Err(FromLuaConversionError { from: "String", to: "ComponentDef", message: Some("Could not determine type".to_owned()) })
                }
            }

            other => Err(FromLuaConversionError { from: other.type_name(), to: "ComponentDef", message: Some("Incorrect type".to_owned()) })
        }
    }
}
