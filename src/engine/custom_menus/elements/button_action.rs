use crate::prelude::*;
use super::parse_from_multiple;
use rlua::{ Value, FromLua, Error::FromLuaConversionError, Table };

#[derive(Clone, Debug)]
pub enum ButtonAction {
    /// Set a value
    SetValue {
        /// What value to set
        key: String,
        /// What to set it to
        value: CustomEventValueType
    },
    /// Perform a custom action (to be handled by a component)
    CustomAction {
        /// Tag of this action
        tag: String,
        /// Value passed on
        value: CustomEventValueType
    },
    /// A regular menu action
    MenuAction(CustomMenuAction),

    /// A conditional 
    Conditional {
        /// The condition to evaluate
        cond: ElementCondition,
        /// What to do if true
        if_true: Box<Self>,
        /// What to do if false
        if_false: Option<Box<Self>>,
    }
}
impl ButtonAction {
    pub fn build(&mut self) {
        match self {
            Self::Conditional { cond, if_true, if_false } => {
                cond.build();
                if_true.build();
                if_false.ok_do_mut(|f| f.build());
            }
            _ => {}
        }
    }

    pub fn resolve(&self, owner: MessageOwner, values: &mut ShuntingYardValues) -> Option<Message> {
        match self {
            Self::MenuAction(action) => {
                if let CustomMenuAction::None = &action { return None };
                let message = MessageType::CustomMenuAction(action.clone());
                Some(Message::new(owner, "", message))
            }
            
            Self::SetValue { key, value } => {
                let val = value.resolve(values)?;
                let action = CustomMenuAction::SetValue(key.clone(), val);
                Some(Message::new(owner, "", MessageType::CustomMenuAction(action)))
            }
            Self::CustomAction { tag, value } => {
                let val = value.resolve(values)?;
                Some(Message::new(owner, tag, MessageType::Value(val)))
            }
            Self::Conditional { cond, if_true, if_false } => {
                match cond.resolve(values) {
                    ElementResolve::Failed | ElementResolve::Error(_) => None,
                    ElementResolve::Unbuilt(_) => unreachable!("conditional element not built!"),
                    ElementResolve::True => if_true.resolve(owner, values),
                    ElementResolve::False => if_false.as_ref().and_then(|f| f.resolve(owner, values)),
                }
            }

        }
    }
}
impl<'lua> FromLua<'lua> for ButtonAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading ButtonAction");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "ButtonAction", message: Some("Not a table".to_owned()) }) };
    
        #[cfg(feature="custom_menu_debugging")] info!("Reading id...");
        let id:String = table.get("id")?;
        #[cfg(feature="custom_menu_debugging")] info!("Got id: {id}");

        match &*id {
            "set_value" => Ok(Self::SetValue {
                key: table.get("key")?,
                value: CustomEventValueType::from_lua(&table)?,
            }),
            "action" => Ok(Self::MenuAction(CustomMenuAction::from_table(&table)?)),
            "custom" => Ok(Self::CustomAction { 
                tag: table.get("tag")?,
                value: CustomEventValueType::from_lua(&table)?
            }),

            "conditional" => Ok(Self::Conditional { 
                cond: ElementCondition::Unbuilt(parse_from_multiple(&table, &["cond", "condition"])?.expect("no condition provided for conditional")),
                if_true: Box::new(table.get("if_true")?), 
                if_false: table.get::<_, Option<ButtonAction>>("if_false")?.map(Box::new)
            }),

            other => Err(FromLuaConversionError { from: "table", to: "ButtonAction", message: Some(format!("unknown id: {other}")) }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CustomEventValueType {
    /// direct value
    Value(CustomElementValue),

    /// get from a variable
    Variable(String),
}

impl CustomEventValueType {
    pub fn resolve(&self, values: &ShuntingYardValues) -> Option<CustomElementValue> {
        match self {
            Self::Value(val) => Some(val.clone()),
            Self::Variable(var) => {
                let val = values.get_raw(var).ok();
                if val.is_none() {
                    error!("custom event value is none! {var}");
                }
                
                val.cloned()
            }
        }
    }

    fn from_lua(table: &Table) -> rlua::Result<Self> {
        if let Some(value) = table.get::<_, Option<CustomElementValue>>("value")? {
            Ok(Self::Value(value))
        } else if let Some(var) = table.get::<_, Option<String>>("variable")? {
            Ok(Self::Variable(var))
        } else { 
            Err(FromLuaConversionError { from: "table", to: "CustomEventValueType", message: Some("not value or variable".to_owned()) })
        }
    }
}


impl<'lua> FromLua<'lua> for CustomElementValue {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="custom_menu_debugging")] info!("Reading CustomElementValue");

        match &lua_value {
            Value::Boolean(b) => Ok(Self::Bool(*b)),
            Value::Integer(i) => Ok(Self::I64(*i)),
            Value::Number(f) => Ok(Self::F32(*f as f32)),
            Value::String(s) => Ok(Self::String(s.to_str()?.to_owned())),
            // Value::Table(table) => {
            //     if let Ok(list) = table.get()
            // }
            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomElementValue", message: None }),
        }

    }
}