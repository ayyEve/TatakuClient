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
    BeatmapList { 
        filter_var: Option<String>, 
    },
}
impl ComponentDef {
    pub async fn build(&self) -> Box<dyn Widgetable> {
        match self {
            Self::BeatmapList { filter_var } => Box::new(BeatmapListComponent::new(filter_var.clone())),
        }
    }
}


impl<'lua> FromLua<'lua> for ComponentDef {
    fn from_lua(lua_value: Value<'lua>, _lua: LuaContext<'lua>) -> Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading ComponentDef");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "ComponentDef", message: Some("Not a table".to_owned()) }) };
    
        let id:String = table.get("id")?;
        #[cfg(feature="custom_menu_debugging")] info!("Is table");
        match &*id {
            "beatmap_list" => {
                Ok(Self::BeatmapList {
                    filter_var: table.get("filter")?,
                })
            }

            _ => Err(FromLuaConversionError { from: "Table", to: "ComponentDef", message: Some("Could not determine type".to_owned()) })
        }
    }
}
