use crate::prelude::*;
use super::parse_from_multiple;
use rlua::{ Value, FromLua, Error::FromLuaConversionError, Table };

// TODO: rename this, its not just used by buttons
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
    },

    Multiple(Vec<Box<Self>>),
}
impl ButtonAction {
    pub fn build(&mut self) {
        match self {
            Self::Conditional { cond, if_true, if_false } => {
                cond.build();
                if_true.build();
                if_false.ok_do_mut(|f| f.build());
            }
            Self::Multiple(list) => list.iter_mut().for_each(|a| a.build()),
            _ => {}
        }
    }

    pub fn resolve(&self, owner: MessageOwner, values: &mut ValueCollection, passed_in: Option<TatakuValue>) -> Option<Message> {
        match self {
            Self::MenuAction(action) => {
                if let CustomMenuAction::None = &action { return None };

                let mut action = action.clone();
                action.build(values);
                let message = MessageType::CustomMenuAction(action, passed_in);
                Some(Message::new(owner, "", message))
            }
            
            Self::SetValue { key, value } => {
                let Some(val) = value.resolve(values, passed_in.clone()) else {
                    warn!("Key doesn't exist: {key}");
                    return None;
                };

                let action = CustomMenuAction::SetValue(key.clone(), val.value);
                Some(Message::new(owner, "", MessageType::CustomMenuAction(action, passed_in)))
            }
            Self::CustomAction { tag, value } => {
                let Some(val) = value.resolve(values, passed_in) else {
                    warn!("Tag doesn't exist: {tag}");
                    return None;
                };

                Some(Message::new(owner, tag, MessageType::Value(val.value)))
            }
            Self::Conditional { cond, if_true, if_false } => {
                match cond.resolve(values) {
                    ElementResolve::Failed | ElementResolve::Error(_) => None,
                    ElementResolve::Unbuilt(_) => unreachable!("conditional element not built!"),
                    ElementResolve::True => if_true.resolve(owner, values, passed_in),
                    ElementResolve::False => if_false.as_ref().and_then(|f| f.resolve(owner, values, passed_in)),
                }
            }

            Self::Multiple(list) => {
                let list = list.iter().map(|a| a.resolve(owner, values, passed_in.clone())).filter_map(|m|m).collect::<Vec<_>>();
                (!list.is_empty()).then(|| Message::new(owner, "", MessageType::Multi(list)))
            }

        }
    }
}
impl<'lua> FromLua<'lua> for ButtonAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading ButtonAction");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "ButtonAction", message: Some("Not a table".to_owned()) }) };

        if let Ok(_) = table.get::<_, Self>(0) {
            let mut list = Vec::new();

            for i in 0.. {
                let Ok(item) = table.get::<_, Self>(i) else { break };
                list.push(Box::new(item));
            }

            return Ok(Self::Multiple(list));
        }
    
        #[cfg(feature="debug_custom_menus")] info!("Reading id...");
        let id:String = table.get("id")?;
        #[cfg(feature="debug_custom_menus")] info!("Got id: {id}");

        match &*id {
            "none" => Ok(Self::MenuAction(CustomMenuAction::None)),
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
    /// No value
    None,

    /// Static value
    Value(TatakuVariable),

    /// Get from a variable
    Variable(String),

    /// Get value from a passed in value
    PassedIn,
}

impl CustomEventValueType {
    pub fn resolve(&self, values: &ValueCollection, passed_in: Option<TatakuValue>) -> Option<TatakuVariable> {
        match self {
            Self::None => None,
            Self::Value(val) => Some(val.clone()),
            Self::Variable(var) => {
                let val = values.get_raw(var).ok();
                if val.is_none() {
                    error!("custom event value is none! {var}");
                }
                
                val.cloned()
            }
            Self::PassedIn => passed_in.map(|v| TatakuVariable::new(v))
        }
    }

    pub fn from_lua(table: &Table) -> rlua::Result<Self> {
        if let Some(value) = table.get::<_, Option<TatakuValue>>("value")? {
            Ok(Self::Value(TatakuVariable::new_any(value)))
        } else if let Some(var) = table.get::<_, Option<String>>("variable")? {
            Ok(Self::Variable(var))
        } else if let Some(_) = table.get::<_, Option<bool>>("passed_in")?{
            Ok(Self::PassedIn)
        } else { 
            Ok(Self::None)
            // Err(FromLuaConversionError { from: "table", to: "CustomEventValueType", message: Some("not value or variable".to_owned()) })
        }
    }
}

impl<'lua> FromLua<'lua> for CustomEventValueType {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        const THIS_TYPE:&str = "CustomEventValueType";
        #[cfg(feature="debug_custom_menus")] info!("Reading {THIS_TYPE}");
        match lua_value {
            Value::Table(table) => Self::from_lua(&table),
            Value::String(s) => Ok(Self::Value(TatakuVariable::new_any(TatakuValue::String(s.to_str()?.to_owned())))),
            Value::Number(n) => Ok(Self::Value(TatakuVariable::new_any(TatakuValue::F32(n as f32)))),
            Value::Integer(n) => Ok(Self::Value(TatakuVariable::new_any(TatakuValue::U64(n as u64)))),

            other =>  Err(FromLuaConversionError { from: other.type_name(), to: THIS_TYPE, message: Some(format!("Bad type")) })
        }
    }
}

impl<'lua> FromLua<'lua> for TatakuValue {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading CustomElementValue");

        match &lua_value {
            Value::Boolean(b) => Ok(Self::Bool(*b)),
            // Value::Integer(i) => Ok(Self::I64(*i)),
            Value::Number(f) => Ok(Self::F32(*f as f32)),
            Value::String(s) => Ok(Self::String(s.to_str()?.to_owned())),
            // Value::Table(table) => {
            //     if let Ok(list) = table.get()
            // }
            other => Err(FromLuaConversionError { from: other.type_name(), to: "CustomElementValue", message: None }),
        }

    }
}

