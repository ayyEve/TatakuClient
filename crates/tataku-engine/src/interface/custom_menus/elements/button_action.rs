use crate::prelude::*;
use super::parse_from_multiple;
use rlua::{ Value, FromLua, Error::FromLuaConversionError, Table };

// TODO: rename this, its not just used by buttons
#[derive(Debug, Clone)]
pub enum LuaAction {
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
impl LuaAction {
    pub fn build(&mut self) {
        match self {
            Self::Conditional { cond, if_true, if_false } => {
                cond.build();
                if_true.build();
                if let Some(i) = if_false {
                    i.build()
                }
            }
            Self::Multiple(list) => list.iter_mut().for_each(|a| a.build()),
            _ => {}
        }
    }

    /// resolve CustomEventValueType::Variable to CustomEventValueType::Value
    pub fn resolve_variables(&mut self, values: &dyn Reflect) {
        match self {
            Self::SetValue { value, ..} => {
                value.resolve_pre(values);
            }
            Self::CustomAction { value, .. } => {
                value.resolve_pre(values);
            }
            Self::Conditional { if_true, if_false, .. } => {
                if_true.resolve_variables(values);
                if let Some(i) = if_false {
                    i.resolve_variables(values);
                }

                // Self::Conditional { cond, if_true, if_false }
            }

            Self::Multiple(list) => {
                list.iter_mut().for_each(|i| i.resolve_variables(values));
                // Self::Multiple(list.into_iter().map(|a| Box::new(a.resolve_pre(values))).collect())
            }

            _ => {}
            // other => other,
        }
    }
    /// resolve CustomEventValueType::PassedIn to CustomEventValueType::Value
    pub fn resolve_passed_in(&self, owner: MessageOwner, passed_in: Option<TatakuValue>) -> Option<Message> {
        self.resolve(owner, &mut DynMap::default(), passed_in)
    }

    pub fn resolve(&self, owner: MessageOwner, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<Message> {
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

                let action = CustomMenuAction::SetValue(key.clone(), val);
                Some(Message::new(owner, "", MessageType::CustomMenuAction(action, passed_in)))
            }
            Self::CustomAction { tag, value } => {
                let Some(val) = value.resolve(values, passed_in) else {
                    warn!("Tag doesn't exist: {tag}");
                    return None;
                };

                Some(Message::new(owner, tag, MessageType::Value(val)))
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
                let list = list.iter().filter_map(|a| a.resolve(owner, values, passed_in.clone())).collect::<Vec<_>>();
                (!list.is_empty()).then(|| Message::new(owner, "", MessageType::Multi(list)))
            }

        }
    }
}
impl<'lua> FromLua<'lua> for LuaAction {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading ButtonAction");
        let Value::Table(table) = lua_value else { return Err(FromLuaConversionError { from: lua_value.type_name(), to: "ButtonAction", message: Some("Not a table".to_owned()) }) };

        if table.get::<_, Self>(0).is_ok() {
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
                if_false: table.get::<_, Option<LuaAction>>("if_false")?.map(Box::new)
            }),

            other => Err(FromLuaConversionError { from: "table", to: "ButtonAction", message: Some(format!("unknown id: {other}")) }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CustomEventValueType {
    /// No value
    None,

    /// Literal value (number, string, bool)
    Value(TatakuValue),

    /// Get from a variable
    Variable(String),

    /// Get value from a passed in value
    PassedIn,
}

impl CustomEventValueType {

    pub fn new_value(value: impl Into<TatakuValue>) -> Self {
        Self::Value(value.into())
    }

    /// pre-emptively resolve variables. used when the element's event requires values to be moved
    pub fn resolve_pre(&mut self, values: &dyn Reflect) {
        if let Self::Variable(var) = self {
            let Ok(val) = values.impl_get(ReflectPath::new(var)) else {
                error!("custom event value is none! {var}");
                *self = Self::None;
                return;
            };
            let value = match TatakuValue::from_reflection(val) {
                Ok(v) => v,
                Err(e) => {
                    error!("custom event value error: {var}, {e:?}");
                    *self = Self::None;
                    return
                }
            };

            *self = Self::Value(value)
        }
    }

    pub fn resolve(&self, values: &dyn Reflect, passed_in: Option<TatakuValue>) -> Option<TatakuValue> {
        match self {
            Self::None => None,
            Self::Value(val) => Some(val.clone()),
            Self::Variable(var) => {
                let Ok(val) = values.impl_get(ReflectPath::new(var)) else {
                    error!("custom event value is none! {var}");
                    return None;
                };
                let value = match TatakuValue::from_reflection(val) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("custom event value error: {var}, {e:?}");
                        return None
                    }
                };

                Some(value)
            }
            Self::PassedIn => passed_in
        }
    }

    pub fn from_lua(table: &Table) -> rlua::Result<Self> {
        if let Some(value) = table.get::<_, Option<TatakuValue>>("value")? {
            Ok(Self::Value(value))
        } else if let Some(var) = table.get::<_, Option<String>>("variable")? {
            Ok(Self::Variable(var))
        } else if table.get::<_, Option<bool>>("passed_in")?.is_some() {
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
            Value::String(s) => Ok(Self::new_value(s.to_str()?.to_owned())),
            Value::Number(n) => Ok(Self::new_value(n as f32)),
            Value::Integer(n) => Ok(Self::new_value(n as u64)),

            other =>  Err(FromLuaConversionError { from: other.type_name(), to: THIS_TYPE, message: Some("Bad type".to_string()) })
        }
    }
}



// pub enum MaybeOwned<'a, T: ?Sized> {
//     Owned(Box<T>),
//     Borrowed(&'a T)
// }
// impl<T: ?Sized> Deref for MaybeOwned<'_, T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::Owned(t) => t,
//             Self::Borrowed(t) => t,
//         }
//     }
// }
