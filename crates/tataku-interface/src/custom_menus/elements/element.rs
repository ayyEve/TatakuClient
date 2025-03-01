use crate::prelude::*;
use crate::prelude::ui::*;
use crate::prelude::lua::*;

#[derive(Clone, Debug)]
pub struct ElementDef {
    pub id: String,
    pub element: ElementIdentifier,
    pub debug_color: Option<Color>,
    pub debug_name: Option<String>,

    pub width: Dimension,
    pub height: Dimension,

    pub style: Style,
}

impl FromLua for ElementDef {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let id = uuid::Uuid::new_v4().to_string();

        #[cfg(feature="debug_custom_menus")] info!("Reading ElementDef");
        let LuaValue::Table(table) = lua_value else { return Err(FromLuaConversionError { 
            from: lua_value.type_name(), 
            to: "ElementIdentifier".to_owned(), 
            message: Some("Not a table".to_owned()) 
        }) };

        let debug_name = table.get::<Option<String>>("debug_name")?;
        #[cfg(feature="debug_custom_menus")]
        debug!("Name: {debug_name:?}");


        let element:String = table.get("id")?;
        #[cfg(feature="debug_custom_menus")] info!("Got id: {id:?}");
        let width  = CustomMenuParser::parse_dimension(table.get("width")?);
        let height = CustomMenuParser::parse_dimension(table.get("height")?);
        let debug_color = table.get("debug_color")?;

        let mut style = Style::default();
        if let Some(value) = read_align_content(table.get("align_content")?) {
            style.align_content = Some(value);
        }
        if let Some(value) = read_align_items(table.get("align_items")?) {
            style.align_items = Some(value);
        }
        if let Some(value) = read_align_content(table.get("justify_content")?) {
            style.justify_content = Some(value);
        }
        if let Some(value) = read_align_items(table.get("justify_items")?) {
            style.justify_items = Some(value);
        }

        if let Some(padding) = table.get::<Option<ElementPadding>>("padding")? {
            style.padding = match padding {
                ElementPadding::Single(n) => taffy::Rect {
                    left: LengthPercentage::Length(n),
                    right: LengthPercentage::Length(n),
                    top: LengthPercentage::Length(n),
                    bottom: LengthPercentage::Length(n),
                },
                ElementPadding::Double([h, v]) => taffy::Rect {
                    left: LengthPercentage::Length(h),
                    right: LengthPercentage::Length(h),
                    top: LengthPercentage::Length(v),
                    bottom: LengthPercentage::Length(v),
                },
                ElementPadding::Quad([t, l, b, r]) => taffy::Rect {
                    top: LengthPercentage::Length(t),
                    left: LengthPercentage::Length(l),
                    bottom: LengthPercentage::Length(b),
                    right: LengthPercentage::Length(r),
                },
            };
        }
        if let Some(margin) = table.get::<Option<ElementPadding>>("margin")? {
            style.margin = match margin {
                ElementPadding::Single(n) => taffy::Rect {
                    left: LengthPercentageAuto::Length(n),
                    right: LengthPercentageAuto::Length(n),
                    top: LengthPercentageAuto::Length(n),
                    bottom: LengthPercentageAuto::Length(n),
                },
                ElementPadding::Double([h, v]) => taffy::Rect {
                    left: LengthPercentageAuto::Length(h),
                    right: LengthPercentageAuto::Length(h),
                    top: LengthPercentageAuto::Length(v),
                    bottom: LengthPercentageAuto::Length(v),
                },
                ElementPadding::Quad([t, l, b, r]) => taffy::Rect {
                    top: LengthPercentageAuto::Length(t),
                    left: LengthPercentageAuto::Length(l),
                    bottom: LengthPercentageAuto::Length(b),
                    right: LengthPercentageAuto::Length(r),
                },
            };
        }


        match &*element {
            "row" => Ok(Self {
                id,
                element: ElementIdentifier::Row {
                    elements: table.get("elements")?,
                    // padding: table.get("padding")?,
                    // margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "col" | "column" => Ok(Self {
                id,
                element: ElementIdentifier::Column {
                    elements: table.get("elements")?,
                    // padding: table.get("padding")?,
                    // margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "dragging_scroll"
            | "drag_scroll"
            | "panel_scroll" => Ok(Self {
                id,
                element: ElementIdentifier::DraggingScroll {
                    elements: table.get("elements")?,
                    // padding: table.get("padding")?,
                    // margin: parse_from_multiple(&table, &["margin", "spacing"])?,
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "space" => Ok(Self {
                id,
                element: ElementIdentifier::Space,
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "text" => Ok(Self {
                id,
                element: ElementIdentifier::Text {
                    text: table.get("text")?,
                    color: table.get("color")?,
                    font_size: table.get("font_size")?,
                    font: table.get("font")?,
                    align: table.get("align")?,
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
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
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "button" => Ok(Self {
                id,
                element: ElementIdentifier::Button {
                    element: Box::new(table.get("element")?),
                    padding: table.get("padding")?,
                    action: table.get("action")?,
                    active_cond: parse_from_multiple(&table, &["cond", "condition", "pressed"])?.map(ElementCondition::Unbuilt),
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "animatable" => Ok(Self {
                id,
                element: ElementIdentifier::Animatable {
                    // TODO: !!
                    triggers: table.get("triggers")?,
                    actions: table.get("actions")?,
                    element: Box::new(table.get("element")?)
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "gameplay_preview" => Ok(Self {
                id,
                element: ElementIdentifier::GameplayPreview { 
                    visualization: table.get("visualization")? 
                },
                style,
                width: width.unwrap_or(FILL),
                height: height.unwrap_or(FILL),
                debug_color,
                debug_name,
            }),

            "styled_content" => Ok(Self {
                id,
                element: ElementIdentifier::StyledContent {
                    element: Box::new(table.get("element")?),
                    // padding: table.get("padding")?,
                    image: table.get("image")?,
                    color: table.get("color")?,
                    border: table.get("border")?,
                    shape: table.get("shape")?,
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "conditional" => Ok(Self {
                id,
                element: ElementIdentifier::Conditional {
                    cond: ElementCondition::Unbuilt(parse_from_multiple(&table, &["cond", "condition"])?.expect("no condition provided for conditional")),
                    if_true: Box::new(table.get("if_true")?),
                    if_false: table.get::<Option<ElementDef>>("if_false")?.map(Box::new),
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
            }),

            "list" => Ok(Self {
                id,
                element: ElementIdentifier::List {
                    list_var: table.get("list")?,
                    scrollable: table.get::<Option<bool>>("scroll")?.unwrap_or_default(),
                    element: Box::new(table.get("element")?),
                    variable: parse_from_multiple(&table, &["var", "variable"])?
                        .ok_or(mlua::Error::FromLuaConversionError {
                            from: "_",
                            to: "list".to_owned(),
                            message: Some("variable parameter not provided".to_string())
                        })?,
                },
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
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
                style,
                width: width.unwrap_or(SHRINK),
                height: height.unwrap_or(SHRINK),
                debug_color,
                debug_name,
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
    fn value_to_float(value: LuaValue) -> LuaResult<f32> {
        match value {
            LuaValue::Integer(i) => Ok(i as f32),
            LuaValue::Number(n) => Ok(n as f32),
            other => Err(FromLuaConversionError { 
                from: other.type_name(), 
                to: "ElementPadding".to_owned(), 
                message: Some("Invalid padding number".to_owned()) 
            }),
        }
    }
}
impl From<ElementPadding> for Padding {
    fn from(value: ElementPadding) -> Self {
        match value {
            ElementPadding::Single(n) => LengthPercentage::Length(n).into(),
            ElementPadding::Double([a, b]) => [
                LengthPercentage::Length(a),
                LengthPercentage::Length(b)
            ].into(),
            ElementPadding::Quad([a, b, c, d]) => [
                LengthPercentage::Length(a),
                LengthPercentage::Length(b),
                LengthPercentage::Length(c),
                LengthPercentage::Length(d),
            ].into(),
        }
    }
}

impl FromLua for ElementPadding {
    fn from_lua(lua_value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading ElementPadding");
        match lua_value {
            LuaValue::Integer(i) => Ok(Self::Single(i as f32)),
            LuaValue::Number(n) => Ok(Self::Single(n as f32)),
            LuaValue::Table(table) => {
                let t = table.get::<Option<LuaValue>>(1)?.map(Self::value_to_float).transpose()?;
                let l = table.get::<Option<LuaValue>>(2)?.map(Self::value_to_float).transpose()?;
                let b = table.get::<Option<LuaValue>>(3)?.map(Self::value_to_float).transpose()?;
                let r = table.get::<Option<LuaValue>>(4)?.map(Self::value_to_float).transpose()?;

                match (t, l, b, r) {
                    (Some(t), None, None, None) => Ok(Self::Single(t)),
                    (Some(t), Some(l), None, None) => Ok(Self::Double([ t, l ])),
                    (Some(t), Some(l), Some(b), Some(r)) => Ok(Self::Quad([ t, l, b, r ])),
                    _ => Err(FromLuaConversionError { 
                        from: "Table", 
                        to: "ElementPadding".to_owned(), 
                        message: Some("Invalid number of table elements for padding".to_owned()) 
                    }),
                }
            }

            other => Err(FromLuaConversionError { 
                from: other.type_name(), 
                to: "ElementPadding".to_owned(), 
                message: Some("Invalid type".to_owned()) 
            })
        }
    }
}



pub(super) fn parse_from_multiple<T:FromLua>(table: &LuaTable, list: &[&'static str]) -> LuaResult<Option<T>> {
    for i in list.iter() {
        #[cfg(feature="debug_custom_menus")] info!("Trying to read value {i}");
        let Some(t) = table.get(*i)? else { continue };
        return Ok(Some(t))
    }

    Ok(None)
}


pub fn read_align_content(value: Option<String>) -> Option<AlignContent> {
    match &*value?.to_lowercase() {
        "start" => Some(AlignContent::Start),
        "end" => Some(AlignContent::End),
        "center" => Some(AlignContent::Center),
        "stretch" => Some(AlignContent::Stretch),
        
        "flex-start" | "flex_start" | "flexstart" => Some(AlignContent::FlexStart),
        "flex-end" | "flex_end" | "flexend" => Some(AlignContent::FlexEnd),

        "space-between" | "space_between" | "spacebetween" => Some(AlignContent::SpaceBetween),
        "space-around" | "space_around" | "spacearound" => Some(AlignContent::SpaceAround),
        "space-evenly" | "space_evenly" | "spaceevenly" => Some(AlignContent::SpaceEvenly),

        other => { 
            warn!("unknown align_content value: {other}");
            None
        }
    }
}
pub fn read_align_items(value: Option<String>) -> Option<taffy::AlignItems> {
    use taffy::AlignItems;
    match &*value?.to_lowercase() {
        "start" => Some(AlignItems::Start),
        "end" => Some(AlignItems::End),
        "center" => Some(AlignItems::Center),
        "stretch" => Some(AlignItems::Stretch),
        "baseline" => Some(AlignItems::Baseline),
        
        "flex_start" | "flexstart" => Some(AlignItems::FlexStart),
        "flex_end" | "flexend" => Some(AlignItems::FlexEnd),

        other => {
            warn!("unknown align_items value: {other}");
            None
        }
    }
}