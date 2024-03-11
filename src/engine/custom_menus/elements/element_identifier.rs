use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum ElementIdentifier {
    Row {
        elements: Vec<ElementDef>,
        padding: Option<ElementPadding>,
        margin: Option<f32>,
    },
    Column {
        elements: Vec<ElementDef>,
        padding: Option<ElementPadding>,
        margin: Option<f32>,
    },
    Space,
    Button {
        element: Box<ElementDef>,
        action: ButtonAction,
        padding: Option<ElementPadding>,
    },
    Text {
        text: CustomElementText,
        color: Option<Color>,
        font_size: Option<f32>,
        font: Option<String>,
    },
    TextInput {
        placeholder: CustomElementText,
        variable: String,
        is_password: bool,
    },

    GameplayPreview {
        visualization: Option<String>,
    },
    Animatable {
        triggers: Vec<AnimatableTrigger>,
        actions: HashMap<String, Vec<AnimatableAction>>,
        element: Box<ElementDef>,
    },
    StyledContent {
        element: Box<ElementDef>,
        padding: Option<ElementPadding>,

        color: Option<Color>,
        border: Option<Border>,
        // image: String,
        shape: Option<Shape>,
    },
    KeyHandler {
        events: Vec<KeyHandlerEvent>,
    },

    Conditional {
        cond: ElementCondition,
        if_true: Box<ElementDef>,
        if_false: Option<Box<ElementDef>>,
    },

    // TODO: !!!
    Custom {

    }
}


#[derive(Clone, Debug)]
pub enum ElementCondition {
    Unbuilt(String),
    Built(Arc<CustomElementCalc>, String),
    Failed,
}


#[derive(Clone, Debug)]
pub struct KeyHandlerEvent {
    pub key: Key,
    pub mods: KeyModifiers,
    pub action: CustomMenuAction,
}
use rlua::{Value, Error, FromLua, Table};
impl<'lua> FromLua<'lua> for KeyHandlerEvent {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::Result<Self> {
        let Value::Table(table) = lua_value else { return Err(Error::FromLuaConversionError { from: "not table", to: "KeyHandlerEvent", message: Some("not a table".to_owned()) }) }; 
        
        let key = table.get("key")?;
        let key = serde_json::from_value(serde_json::Value::String(key))
            .map_err(|e| Error::FromLuaConversionError { from: "String", to: "Key", message: Some(e.to_string()) })?;

        let mut mods = KeyModifiers::default();
        let mods_table = table.get::<_, Table>("mods")?;
        for i in 0..3 { 
            let Some(a) = mods_table.get::<_, Option<String>>(i)? else { break };
            match &*a {
                "ctrl" | "control" => mods.ctrl = true,
                "alt" => mods.alt = true,
                "shift" => mods.shift = true,
                _ => {}
            }
        }

        Ok(KeyHandlerEvent {
            key,
            mods,
            action: table.get("action")?
        })
    }
}
