use crate::prelude::*;
use crate::prelude::iced_elements::*;

use rlua::{Value, Error, FromLua};

#[derive(Clone, Debug)]
pub struct ElementDef {
    pub id: ElementIdentifier,
    pub width: Length,
    pub height: Length,
}
impl ElementDef {
    #[async_recursion::async_recursion]
    pub async fn build(&self) -> BuiltElementDef {
        let mut built = BuiltElementDef {
            element: self.clone(),
            children: Vec::new(),
        };

        match &mut built.element.id {
            ElementIdentifier::GameplayPreview { visualization } => {
                let mut gameplay = GameplayPreview::new(true, true, Arc::new(|_|true));
                gameplay.widget.width(self.width);
                gameplay.widget.height(self.height);
                
                if let Some(vis) = visualization {
                    match &**vis {
                        "menu_visualization" => gameplay.visualization = Some(Box::new(MenuVisualization::new().await)),
                        _ => warn!("Unknown gameplay visualization: {vis}"),
                    }
                }

                built.children.push(Box::new(gameplay));
            }
            ElementIdentifier::Column { elements, .. }
            | ElementIdentifier::Row { elements, .. } 
                => for i in elements.iter_mut() {
                    built.children.push(Box::new(i.build().await))
                }

            ElementIdentifier::Animatable { triggers:_, actions:_, element }
                => built.children.push(Box::new(element.build().await)),

            ElementIdentifier::StyledContent { element, .. } 
                => built.children.push(Box::new(element.build().await)),
            
            ElementIdentifier::Button { element, .. } 
                => built.children.push(Box::new(element.build().await)),

            ElementIdentifier::Text { text, .. } => {
                if let Err(e) = text.parse() {
                    error!("error building custom text: {e:?}");
                }
            }

            ElementIdentifier::Conditional { cond, if_true, if_false } => {
                let ElementCondition::Unbuilt(s) = cond else { unreachable!() };
                match CustomElementCalc::parse(format!("{s} == true")) {
                    Ok(built) => *cond = ElementCondition::Built(Arc::new(built), s.clone()),
                    Err(e) => {
                        error!("Error building conditional: {e:?}");
                        *cond = ElementCondition::Failed;
                    }
                }

                built.children.push(Box::new(if_true.build().await));
                if let Some(if_false) = if_false {
                    built.children.push(Box::new(if_false.build().await));
                }
            }

            _ => {},
        }

        built
    }
}

impl<'lua> FromLua<'lua> for ElementDef {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::Result<Self> {
        let Value::Table(table) = lua_value else { return Err(Error::FromLuaConversionError { from: lua_value.type_name(), to: "ElementIdentifier", message: Some("Not a table".to_owned()) }) };
    
        let id:String = table.get("id")?;
        let width = CustomMenuParser::parse_length(table.get("width")?);
        let height = CustomMenuParser::parse_length(table.get("height")?);

        match &*id {
            "row" => Ok(Self {
                id: ElementIdentifier::Row { 
                    elements: table.get("elements")?,
                    padding: table.get("padding")?,
                    margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),
            
            "col" | "column" => Ok(Self {
                id: ElementIdentifier::Column { 
                    elements: table.get("elements")?,
                    padding: table.get("padding")?,
                    margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),
            
            "space" => Ok(Self {
                id: ElementIdentifier::Space,
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "text" => Ok(Self {
                id: ElementIdentifier::Text { 
                    text: table.get("text")?, 
                    color: table.get("color")?, 
                    font_size: table.get("font_size")?,
                    font: table.get("font")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }), 

            "button" => Ok(Self {
                id: ElementIdentifier::Button { 
                    element: Box::new(table.get("element")?),
                    padding: table.get("padding")?,
                    action: table.get("action")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "animatable" => Ok(Self {
                id: ElementIdentifier::Animatable {
                    // TODO: !! 
                    triggers: Default::default(),
                    actions: Default::default(),
                    element: Box::new(table.get("element")?) 
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "gameplay_preview" => Ok(Self {
                id: ElementIdentifier::GameplayPreview { visualization: table.get("visualization")? },
                width: width.unwrap_or(Length::Fill),
                height: height.unwrap_or(Length::Fill),
            }),
            

            "music_player" => Ok(Self {
                id: ElementIdentifier::Space, // { display: CurrentSongDisplay::new() },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "styled_content" => Ok(Self {
                id: ElementIdentifier::StyledContent { 
                    element: Box::new(table.get("element")?),
                    padding: table.get("padding")?,

                    color: table.get("color")?,
                    border: table.get("border")?,
                    shape: table.get("shape")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "conditional" => Ok(Self {
                id: ElementIdentifier::Conditional { 
                    cond: ElementCondition::Unbuilt(parse_from_multiple(&table, &["cond", "condition"])?.expect("no condition provided for conditional")),
                    if_true: Box::new(table.get("if_true")?),
                    if_false: table.get::<_, Option<ElementDef>>("if_false")?.map(Box::new),
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "key_handler" => {
                let table = table.get::<_, rlua::Table>("events")?;
                Ok(Self {
                    id: ElementIdentifier::KeyHandler { 
                        events: (0..30).into_iter().filter_map(|i|table.get(i).ok()).collect()
                    },
                    width: Length::Fixed(0.0),
                    height: Length::Fixed(0.0)
                })
            }
            
            _ => { todo!("{id}") }
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum ElementPadding {
    Single(f32),
    Double([f32; 2]),
    Quad([f32; 4])
}
impl ElementPadding {
    fn value_to_float<'lua>(value: Value<'lua>) -> rlua::Result<f32> {
        match value {
            Value::Integer(i) => Ok(i as f32),
            Value::Number(n) => Ok(n as f32),
            other => Err(Error::FromLuaConversionError { from: other.type_name(), to: "ElementPadding", message: Some("Invalid padding number".to_owned()) }),
        }
    }
}
impl Into<iced::Padding> for ElementPadding {
    fn into(self) -> iced::Padding {
        match self {
            Self::Single(f) => iced::Padding::new(f),
            Self::Double(a) => iced::Padding::from(a),
            Self::Quad(a) => iced::Padding::from(a),
        }
    }
}
impl<'lua> FromLua<'lua> for ElementPadding {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::prelude::LuaContext<'lua>) -> rlua::Result<Self> {
        match lua_value {
            Value::Integer(i) => Ok(Self::Single(i as f32)),
            Value::Number(n) => Ok(Self::Single(n as f32)),
            Value::Table(table) => {
                let t = table.get::<_, Option<Value>>(1)?.map(Self::value_to_float).transpose()?;
                let l = table.get::<_, Option<Value>>(2)?.map(Self::value_to_float).transpose()?;
                let b = table.get::<_, Option<Value>>(3)?.map(Self::value_to_float).transpose()?;
                let r = table.get::<_, Option<Value>>(4)?.map(Self::value_to_float).transpose()?;

                match (t, l, b, r) {
                    (Some(t), None, None, None) => Ok(Self::Single(t)),
                    (Some(t), Some(l), None, None) => Ok(Self::Double([ t, l ])),
                    (Some(t), Some(l), Some(b), Some(r)) => Ok(Self::Quad([ t, l, b, r ])),
                    _ => Err(Error::FromLuaConversionError { from: "Table", to: "ElementPadding", message: Some("Invalid number of table elements for padding".to_owned()) }),
                }
            }

            other => Err(Error::FromLuaConversionError { from: other.type_name(), to: "ElementPadding", message: Some("Invalid type".to_owned()) })
        }
    }
}



fn parse_from_multiple<'lua, T:FromLua<'lua>>(table: &rlua::Table<'lua>, list: &[&'static str]) -> rlua::Result<Option<T>> {
    for i in list.iter() {
        let Some(t) = table.get(*i)? else { continue };
        return Ok(Some(t))
    }

    Ok(None)
}