use crate::prelude::*;
use crate::prelude::iced_elements::*;
use rlua::{ Value, Error, FromLua };
use super::super::parsers::LuaColor;

#[derive(Clone, Debug)]
pub struct ElementDef {
    pub id: String,
    pub element: ElementIdentifier,
    pub debug_color: Option<Color>,
    pub width: Length,
    pub height: Length,
}
impl ElementDef {
    #[async_recursion::async_recursion]
    pub async fn build(&self, skin_manager: &mut dyn SkinProvider, owner: MessageOwner) -> Box<BuiltElementDef> {
        let mut built = BuiltElementDef {
            element: self.clone(),
            children: Vec::new(),
        };

        match &mut built.element.element {
            ElementIdentifier::GameplayPreview { visualization } => {
                let mut gameplay = GameplayPreview::new(true, true, Arc::new(|_| true), owner)
                    .width(self.width)
                    .height(self.height);
                gameplay.reload_skin(skin_manager).await;
                
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
            | ElementIdentifier::PanelScroll { elements, .. }
                => for i in elements.iter() {
                    built.children.push(i.build(skin_manager, owner).await);
                }

            ElementIdentifier::Animatable { triggers:_, actions:_, element }
                => built.children.push(element.build(skin_manager, owner).await),

            ElementIdentifier::StyledContent { element, image, built_image, .. } 
                => {
                    if let Some(image) = image {
                        *built_image = skin_manager.get_texture(image, &TextureSource::Skin, SkinUsage::Game, true).await;
                    }
                    built.children.push(element.build(skin_manager, owner).await)
                },
            
            ElementIdentifier::Button { element, action, .. } 
                => {
                    action.build();
                    built.children.push(element.build(skin_manager, owner).await)
                }

            ElementIdentifier::Text { text, .. } => {
                if let Err(e) = text.parse() {
                    error!("error building custom text: {e:?}");
                }
            }

            ElementIdentifier::TextInput { on_input, on_submit, .. } => {
                if let Some(i) = on_input.as_mut() { i.build() }
                if let Some(i) = on_submit.as_mut() { i.build() }
            }

            ElementIdentifier::Conditional { cond, if_true, if_false } => {
                cond.build();

                built.children.push(if_true.build(skin_manager, owner).await);
                if let Some(if_false) = if_false {
                    built.children.push(if_false.build(skin_manager, owner).await);
                }
            }

            ElementIdentifier::List { element, .. } => {
                built.children.push(element.build(skin_manager, owner).await);
            }

            ElementIdentifier::Dropdown { on_select, .. } => {
                if let Some(a) = on_select.as_mut() { a.build() }
            }

            _ => {},
        }

        for e in built.children.iter_mut() {
            e.reload_skin(skin_manager);
        }

        Box::new(built)
    }
}

impl<'lua> FromLua<'lua> for ElementDef {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();

        #[cfg(feature="debug_custom_menus")] info!("Reading ElementDef");
        let Value::Table(table) = lua_value else { return Err(Error::FromLuaConversionError { from: lua_value.type_name(), to: "ElementIdentifier", message: Some("Not a table".to_owned()) }) };
        
        
        #[cfg(feature="debug_custom_menus")] 
        table.get::<_, Option<String>>("debug_name")?.ok_do(|name| debug!("Name: {name}"));
        

        let element:String = table.get("id")?;
        #[cfg(feature="debug_custom_menus")] info!("Got id: {id:?}");
        let width = CustomMenuParser::parse_length(table.get("width")?);
        let height = CustomMenuParser::parse_length(table.get("height")?);
        let debug_color = table.get::<_, Option<LuaColor>>("debug_color")?.map(|c| c.0);

        match &*element {
            "row" => Ok(Self {
                id,
                element: ElementIdentifier::Row { 
                    elements: table.get("elements")?,
                    padding: table.get("padding")?,
                    margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),
            
            "col" | "column" => Ok(Self {
                id,
                element: ElementIdentifier::Column { 
                    elements: table.get("elements")?,
                    padding: table.get("padding")?,
                    margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),

            "panel_scroll" => Ok(Self {
                id,
                element: ElementIdentifier::PanelScroll { 
                    elements: table.get("elements")?,
                    padding: table.get("padding")?,
                    margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),
            
            "space" => Ok(Self {
                id,
                element: ElementIdentifier::Space,
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),

            "text" => Ok(Self {
                id,
                element: ElementIdentifier::Text { 
                    text: table.get("text")?, 
                    color: table.get::<_, Option<LuaColor>>("color")?.map(|c| c.0), 
                    font_size: table.get("font_size")?,
                    font: table.get("font")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }), 

            "text_input" => Ok(Self {
                id,
                element: ElementIdentifier::TextInput { 
                    placeholder: table.get("placeholder")?, 
                    variable: table.get("variable")?, 
                    on_input: table.get("on_input")?,
                    on_submit: table.get("on_submit")?,
                    is_password: parse_from_multiple(&table, &["is_password", "password"])?.unwrap_or_default(),
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }), 

            "button" => Ok(Self {
                id,
                element: ElementIdentifier::Button { 
                    element: Box::new(table.get("element")?),
                    padding: table.get("padding")?,
                    action: table.get("action")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),

            "animatable" => Ok(Self {
                id,
                element: ElementIdentifier::Animatable {
                    // TODO: !! 
                    triggers: Default::default(),
                    actions: Default::default(),
                    element: Box::new(table.get("element")?) 
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),

            "gameplay_preview" => Ok(Self {
                id,
                element: ElementIdentifier::GameplayPreview { visualization: table.get("visualization")? },
                width: width.unwrap_or(Length::Fill),
                height: height.unwrap_or(Length::Fill),
                debug_color,
            }),

            "styled_content" => Ok(Self {
                id,
                element: ElementIdentifier::StyledContent { 
                    element: Box::new(table.get("element")?),
                    padding: table.get("padding")?,
                    image: table.get("image")?,
                    built_image: None,

                    color: table.get::<_, Option<LuaColor>>("color")?.map(|c| c.0),
                    border: table.get("border")?,
                    shape: table.get("shape")?,
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),

            "conditional" => Ok(Self {
                id,
                element: ElementIdentifier::Conditional { 
                    cond: ElementCondition::Unbuilt(parse_from_multiple(&table, &["cond", "condition"])?.expect("no condition provided for conditional")),
                    if_true: Box::new(table.get("if_true")?),
                    if_false: table.get::<_, Option<ElementDef>>("if_false")?.map(Box::new),
                },
                width: width.unwrap_or(Length::Shrink),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),

            "list" => Ok(Self {
                id,
                element: ElementIdentifier::List { 
                    list_var: table.get("list")?,
                    scrollable: table.get::<_, Option<bool>>("scroll")?.unwrap_or_default(),
                    element: Box::new(table.get("element")?),
                    variable: parse_from_multiple(&table, &["var", "variable"])?
                        .ok_or(rlua::Error::FromLuaConversionError { 
                            from: "_", 
                            to: "list", 
                            message: Some("variable parameter not provided".to_string())
                        })?,
                },
                width: width.unwrap_or(Length::Fill),
                height: height.unwrap_or(Length::Shrink),
                debug_color,
            }),
            
            "dropdown" => Ok(Self {
                id,
                element: ElementIdentifier::Dropdown { 
                    options_key: table.get("options_key")?, 
                    options_display_key: table.get("options_display_key")?, 
                    selected_key: table.get("selected_key")?, 
                    on_select: table.get("on_select")?, 
                    padding: table.get("padding")?, 
                    placeholder: table.get("placeholder")?, 
                    font_size: table.get("font_size")?, 
                    font: table.get("font")?
                },
                width: width.unwrap_or(Length::Fill),
                height: height.unwrap_or(Length::Shrink), // not actually used 
                debug_color,
            }),
            
            _ => { todo!("{element}") }
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
    fn value_to_float(value: Value<'_>) -> rlua::Result<f32> {
        match value {
            Value::Integer(i) => Ok(i as f32),
            Value::Number(n) => Ok(n as f32),
            other => Err(Error::FromLuaConversionError { from: other.type_name(), to: "ElementPadding", message: Some("Invalid padding number".to_owned()) }),
        }
    }
}
impl From<ElementPadding> for iced::Padding {
    fn from(val: ElementPadding) -> Self {
        match val {
            ElementPadding::Single(f) => iced::Padding::new(f),
            ElementPadding::Double(a) => iced::Padding::from(a),
            ElementPadding::Quad(a) => iced::Padding::from(a),
        }
    }
}
impl<'lua> FromLua<'lua> for ElementPadding {
    fn from_lua(lua_value: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading ElementPadding");
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



pub(super) fn parse_from_multiple<'lua, T:FromLua<'lua>>(table: &rlua::Table<'lua>, list: &[&'static str]) -> rlua::Result<Option<T>> {
    for i in list.iter() {
        #[cfg(feature="debug_custom_menus")] info!("Trying to read value {i}");
        let Some(t) = table.get(*i)? else { continue };
        return Ok(Some(t))
    }

    Ok(None)
}
