use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct AnimatableTrigger {
    pub trigger: AnimatableTriggerEvent,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimatableTriggerEvent {
    Input,
    NoInput { duration: f32 },
    Hover,
    Unhover,
    Click,
    ClickHold { duration: f32 }, 
    Unclick,

    Event(TatakuEventType),
    Message(MessageTag)
}

#[derive(Clone, Debug)]
pub struct AnimatableAction {
    pub action: TransformType,
    pub start: AnimatableTransformValue,
    pub stop: AnimatableTransformValue,
    pub duration: f32,
}

// TODO: only use val? other variants arent gonna be super helpful
#[derive(Clone, Debug)]
pub enum AnimatableTransformValue {
    Val(f32),
    Current,
    ParentWidth,
    ParentHeight,
}


mod lua {
    use crate::prelude::*;
    use crate::prelude::lua::*;

    impl FromLua for AnimatableTrigger {
        fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
            let LuaValue::Table(table) = value else {
                return Err(FromLuaConversionError { 
                    from: value.type_name(), 
                    to: "AnimatableTrigger".to_owned(), 
                    message: None,
                });
            };

            let action = table.get("action")?;
            let trigger = AnimatableTriggerEvent::from_lua(LuaValue::Table(table), lua)?;

            Ok(Self {
                trigger,
                action,
            })
        }
    }

    impl FromLua for AnimatableTriggerEvent {
        fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
            match value {
                LuaValue::Table(table) => {
                    let id = table.get::<String>("trigger")?;

                    match &*id {
                        "input" => Ok(Self::Input),
                        "hover" => Ok(Self::Hover),
                        "unhover" => Ok(Self::Unhover),
                        "click" => Ok(Self::Click),
                        "unclick" | "release" => Ok(Self::Unclick),

                        "clickhold" | "click_hold" | "hold" => Ok(Self::ClickHold { 
                            duration: table.get("duration")? 
                        }),
                        "no_input" | "noinput" => Ok(Self::NoInput { 
                            duration: table.get("duration")? 
                        }),

                        "event" => Ok(Self::Event(table.get("event")?)),

                        other => Err(FromLuaConversionError { 
                            from: "Table", 
                            to: "AnimatableTriggerEvent".to_owned(), 
                            message: Some(format!("unknown event: {other}")),
                        })
                    }
                }
                LuaValue::String(str) => {
                    match &*str.to_string_lossy() {
                        "input" => Ok(Self::Input),
                        "hover" => Ok(Self::Hover),
                        "unhover" => Ok(Self::Unhover),
                        "click" => Ok(Self::Click),
                        "unclick" | "release" => Ok(Self::Unclick),

                        other => Err(FromLuaConversionError { 
                            from: "String", 
                            to: "AnimatableTriggerEvent".to_owned(), 
                            message: Some(format!("unknown event: {other}")),
                        })
                    }
                }

                other => Err(FromLuaConversionError { 
                    from: other.type_name(), 
                    to: "AnimatableTriggerEvent".to_owned(), 
                    message: None,
                })
            }
        }
    }



    impl FromLua for AnimatableAction {
        fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
            let LuaValue::Table(table) = value else {
                return Err(FromLuaConversionError { 
                    from: value.type_name(), 
                    to: "AnimatableAction".to_owned(), 
                    message: None,
                });
            };

            let duration = table.get::<f32>("duration")?;
            match &*table.get::<String>("action")? {
                // "vector_scale" => Ok(AnimatableAction {
                //     action: TransformType::VectorScale { 
                //         start: table.get("start")?, 
                //         end: table.get("stop")?,
                //     },
                //     start: AnimatableTransformValue::Current,
                //     stop: AnimatableTransformValue::Current,
                //     duration,
                // }),
                "scale" => Ok(AnimatableAction {
                    action: TransformType::Scale { 
                        start: table.get("start")?, 
                        end: table.get("stop")?,
                    },
                    start: AnimatableTransformValue::Current,
                    stop: AnimatableTransformValue::Current,
                    duration,
                }),
                "scale_x" => Ok(AnimatableAction {
                    action: TransformType::ScaleX { 
                        start: table.get("start")?, 
                        end: table.get("stop")?,
                    },
                    start: AnimatableTransformValue::Current,
                    stop: AnimatableTransformValue::Current,
                    duration,
                }),
                "scale_y" => Ok(AnimatableAction {
                    action: TransformType::ScaleX { 
                        start: table.get("start")?, 
                        end: table.get("stop")?,
                    },
                    start: AnimatableTransformValue::Current,
                    stop: AnimatableTransformValue::Current,
                    duration,
                }),
                "rotation" => Ok(AnimatableAction {
                    action: TransformType::Rotation { 
                        start: table.get("start")?, 
                        end: table.get("stop")?,
                    },
                    start: AnimatableTransformValue::Current,
                    stop: AnimatableTransformValue::Current,
                    duration,
                }),
                // "position" => Ok(AnimatableAction {
                //     action: TransformType::Position { 
                //         start: table.get("start")?, 
                //         end: table.get("stop")?,
                //     },
                //     start: AnimatableTransformValue::Current,
                //     stop: AnimatableTransformValue::Current,
                //     duration,
                // }),
                "position_x" => Ok(AnimatableAction {
                    action: TransformType::PositionX { 
                        start: table.get("start")?, 
                        end: table.get("stop")?,
                    },
                    start: AnimatableTransformValue::Current,
                    stop: AnimatableTransformValue::Current,
                    duration,
                }),
                "position_y" => Ok(AnimatableAction {
                    action: TransformType::PositionY { 
                        start: table.get("start")?, 
                        end: table.get("stop")?,
                    },
                    start: AnimatableTransformValue::Current,
                    stop: AnimatableTransformValue::Current,
                    duration,
                }),

                other => Err(FromLuaConversionError { 
                    from: "Table", 
                    to: "AnimatableAction".to_owned(), 
                    message: Some(format!("unknown transform type: {other}")),
                }),
            }


            // #[derive(Copy, Clone, Debug, Default)]
            // pub enum TransformType {
            //     Color { start: Color, end: Color },
            //     BorderSize { start: f32, end: f32 },
            //     Transparency { start: f32, end: f32 },
            //     Position { start: Vector2, end: Vector2 },
            //     PositionX { start: f32, end: f32 },
            //     PositionY { start: f32, end: f32 },
            //     BorderTransparency { start: f32, end: f32 },
            // }

        }
    }



    
}
