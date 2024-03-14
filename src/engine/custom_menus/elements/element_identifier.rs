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
        image: Option<String>,
        built_image: Option<Image>,
        shape: Option<Shape>,
    },

    //TODO: move this to a component
    KeyHandler {
        events: Vec<KeyHandlerEvent>,
    },

    Conditional {
        cond: ElementCondition,
        if_true: Box<ElementDef>,
        if_false: Option<Box<ElementDef>>,
    },

    List {
        list_var: String,
        scrollable: bool,
        element: Box<ElementDef>,
        variable: Option<String>,
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
impl ElementCondition {
    pub fn build(&mut self) {
        let ElementCondition::Unbuilt(s) = self else { unreachable!() };
        match CustomElementCalc::parse(format!("{s} == true")) {
            Ok(built) => *self = ElementCondition::Built(Arc::new(built), s.clone()),
            Err(e) => {
                error!("Error building conditional: {e:?}");
                *self = ElementCondition::Failed;
            }
        }
    }

    pub fn resolve<'a>(&'a self, values: &ShuntingYardValues) -> ElementResolve<'a> {
        match self {
            Self::Failed => ElementResolve::Failed,
            Self::Unbuilt(calc_str) => ElementResolve::Unbuilt(calc_str),
            Self::Built(calc, calc_str) => {
                match calc.resolve(values).map(|n| n.as_bool()) {
                    Ok(true) => ElementResolve::True,
                    Ok(false) => ElementResolve::False,
                    Err(e) => {
                        error!("Error with shunting yard calc. calc: '{calc_str}', error: {e:?}");
                        ElementResolve::Error(e)
                    }
                }
            }
        }
    }
}
pub enum ElementResolve<'a> {
    Failed,
    Unbuilt(&'a String),
    True,
    False,
    Error(ShuntingYardError)
}


#[derive(Clone, Debug)]
pub struct KeyHandlerEvent {
    pub key: Key,
    pub mods: KeyModifiers,
    pub action: ButtonAction,
}
impl KeyHandlerEvent {
    pub fn build(&mut self) {
        self.action.build();
    }
}

use rlua::{ Value, Error::FromLuaConversionError, FromLua };
impl<'lua> FromLua<'lua> for KeyHandlerEvent {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading KeyhandlerEvent");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "KeyHandlerEvent", message: None }) }; 
        
        #[cfg(feature="custom_menu_debugging")] info!("Reading key");
        let key = table.get("key")?;
        let key = serde_json::from_value(serde_json::Value::String(key))
            .map_err(|e| FromLuaConversionError { from: "String", to: "Key", message: Some(e.to_string()) })?;

        #[cfg(feature="custom_menu_debugging")] info!("Reading mods");
        let mut mods = KeyModifiers::default();
        if let Some(incoming_mods) = table.get::<_, Option<Vec<String>>>("mods")? {
            for m in incoming_mods { 
                match &*m {
                    "ctrl" | "control" => mods.ctrl = true,
                    "alt" => mods.alt = true,
                    "shift" => mods.shift = true,
                    _ => {}
                }
            }
        }

        Ok(KeyHandlerEvent {
            key,
            mods,
            action: table.get("action")?
        })
    }
}
