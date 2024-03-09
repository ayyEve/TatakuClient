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
            special: BuiltElementData::None,
        };

        match &self.id {
            ElementIdentifier::SongDisplay => built.special = BuiltElementData::SongDisplay(CurrentSongDisplay::new()),
            ElementIdentifier::GameplayPreview { visualization } => {
                let mut gameplay = GameplayPreview::new(true, true, Arc::new(|_|true));
                if let Some(vis) = visualization {
                    match &**vis {
                        "menu_visualization" => gameplay.visualization = Some(Box::new(MenuVisualization::new().await)),
                        _ => warn!("Unknown gameplay visualization: {vis}"),
                    }
                }
                built.special = BuiltElementData::GameplayPreview(gameplay)
            }
            ElementIdentifier::Column { elements, .. }
            | ElementIdentifier::Row { elements, .. } => built.special = BuiltElementData::Elements(BuiltElementDef::build_elements(elements).await),

            ElementIdentifier::Animatable { triggers:_, actions:_, element }
             => built.special = BuiltElementData::Element(Box::new(element.build().await)),

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
                    margin: table.get("margin")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),
            
            "col" | "column" => Ok(Self {
                id: ElementIdentifier::Column { 
                    elements: table.get("elements")?,
                    padding: table.get("padding")?,
                    margin: table.get("margin")?,
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
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }), 

            "button" => Ok(Self {
                id: ElementIdentifier::Button { 
                    text: table.get("text")?, 
                    color: table.get("color")?, 
                    font_size: table.get("font_size")?,
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
            

            "song_display" => Ok(Self {
                id: ElementIdentifier::SongDisplay,
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),

            "music_player" => Ok(Self {
                id: ElementIdentifier::Space, // { display: CurrentSongDisplay::new() },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
            }),


            
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