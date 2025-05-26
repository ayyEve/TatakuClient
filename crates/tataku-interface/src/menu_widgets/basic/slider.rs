use crate::prelude::*;
use crate::prelude::ui::*;
use std::ops::RangeInclusive;


#[derive(Widget)]
// TODO: should we make this generic? maybe use a ReflectNumber instead of f32?
#[derive(ChainableInitializer)]
pub struct Slider {
    #[chain] pub style: Style,

    pub value: SliderValue,
    pub range: RangeInclusive<f32>,
    #[chain] pub step: Option<f32>,

    pub on_change: InputAction<f32>, //SliderOnChange,

    hovered: bool,
    // active: bool,
    pressed: bool,
    node_id: NodeId,
}
impl Slider {
    pub fn new(
        range: RangeInclusive<f32>,
        value: impl Into<SliderValue>,
        // on_change: impl Into<SliderOnChange>,
        on_change: impl Into<InputAction<f32>>,
    ) -> Self {
        Self {
            style: Style {
                size: Size { 
                    width: Dimension::Percent(0.25), 
                    height: Dimension::Percent(1.0),
                },
                ..Style::default()
            },

            range,
            value: value.into(),
            step: None,
            on_change: on_change.into(),
            
            hovered: false,
            pressed: false,
            node_id: EMPTY_NODE,
        }
    }
}
impl Widget for Slider {
    fn name(&self) -> Cow<'static, str> { "slider_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf(self.style.clone())?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        let Some(ctx) = shell.tree.get_context(self.node_id) 
        else { return };
        let active = ctx.selected.unwrap();

        match &event.event {
            InputType::MouseMove(pos) => {
                let pos = ctx.inverse_global_transform * *pos;
                let bounds = shell.tree
                    .content_bounds(self.node_id)
                    .unwrap();
                self.hovered = bounds.contains(pos);

                if self.pressed {
                    let value = self.value.get();
                    let start = *self.range.start();
                    let end = *self.range.end();

                    let percent = (pos.x - bounds.pos.x) / bounds.size.x;
                    let mut new_value = (start + percent * (end - start))
                        .clamp(start, end);

                    if let Some(snap) = self.step {
                        // apply_snap
                        if let Some(val) = apply_snap(
                            &self.range, 
                            value, 
                            new_value, 
                            snap
                        ) { 
                            new_value = val;
                        } else {
                            new_value = value;
                        }
                    }

                    if (value - new_value).abs() > f32::EPSILON {
                        self.value.set(new_value);
                        self.on_change.run(
                            &new_value,
                            shell.owner,
                            shell.messages,
                            shell.actions,
                            shell.values,
                        );
                        // if let Some(msg) = self.on_change.resolve(
                        //     new_value,
                        //     shell.owner,
                        //     shell.values
                        // ) {
                        //     shell.messages.push(msg);
                        // }
                    }
                }
            }

            InputType::MousePress(MouseButton::Left) if self.hovered => {
                self.pressed = true;
                shell.event_consumed = true;
            }
            InputType::MouseRelease(MouseButton::Left) => {
                self.pressed = false;
                // never consume a mouse release event
            }

            InputType::KeyPress(press) => {
                let Some(key) = press.as_key() else { return };

                match key {
                    Key::Left => if active || self.hovered {
                        shell.event_consumed = true;
                        self.value.set((self.value.get() - self.step
                            .unwrap_or(1.0))
                            .clamp(*self.range.start(), *self.range.end())
                        );
                    }
                    Key::Right => if active || self.hovered {
                        shell.event_consumed = true;
                        self.value.set(
                            (self.value.get() + self.step.unwrap_or(1.0))
                            .clamp(*self.range.start(), *self.range.end())
                        );
                    }

                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.value.update(shell.values);
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        shell.list.push(
            Rectangle::new_bounds(bounds, Color::TRANSPARENT)
                .border(Border::new(Color::PUMPKIN_ORANGE, 2.0))
        );

        // draw track
        let track = Bounds::new(
            Vector2::new(
                bounds.pos.x,
                bounds.pos.y + bounds.size.y / 2.0 - 2.0,
            ),
            Vector2::new(
                bounds.size.y,
                4.0
            )
        );

        shell.list.push(Rectangle::new_bounds(bounds, Color::BLACK));

        // draw slider
        let start = *self.range.start();
        let end = *self.range.end();
        let percent = (self.value.get() - start) / end;

        let dragger_pos = Vector2::new(
            bounds.pos.x + bounds.size.x * percent,
            track.pos.y
        );

        shell.list.push(Circle::new(
            dragger_pos,
            (bounds.size.y / 2.0) * 5.0/6.0,
            shell.general_theme.default_color
        ).border(Border::new(
                if self.pressed {
                    shell.general_theme.active_color
                } else if self.hovered {
                    shell.general_theme.hover_color
                } else {
                    shell.general_theme.default_color
                },
                2.0
            ))
        );
    }
}


pub enum SliderValue {
    Static(f32),
    Variable {
        variable: String,
        value: f32,
    },
    Error,
}
impl SliderValue {
    fn get(&self) -> f32 {
        match self {
            Self::Error => 0.0,
            Self::Static(n) => *n,
            Self::Variable { value, .. } => *value,
        }
    }
    fn set(&mut self, new: f32) {
        match self {
            Self::Error => {},
            Self::Static(v) => *v = new,
            Self::Variable { value, .. } => *value = new,
        }
    }

    fn update(&mut self, values: &dyn Reflect) {
        match self {
            Self::Static(_) | Self::Error => {},
            Self::Variable {
                variable,
                value
            }=> match values.reflect_as_number(&*variable) {
                Ok(n) => *value = n.into(),
                Err(e) => {
                    warn!("error with get: {e:?}");
                    *self = Self::Error;
                }
            }
        }
    }
}
impl From<f32> for SliderValue {
    fn from(value: f32) -> Self {
        Self::Static(value)
    }
}
impl From<String> for SliderValue {
    fn from(variable: String) -> Self {
        Self::Variable {
            variable,
            value: 0.0,
        }
    }
}
impl From<SliderBuilderValue> for SliderValue {
    fn from(value: SliderBuilderValue) -> Self {
        match value {
            SliderBuilderValue::Static(n) => Self::Static(n),
            SliderBuilderValue::Variable(v) => Self::Variable {
                variable: v,
                value: 0.0
            },
        }
    }
}


impl From<SliderBuilderOnChange> for InputAction<f32> {
    fn from(value: SliderBuilderOnChange) -> Self {
        match value {
            SliderBuilderOnChange::Message(m) => Self::Message(m),
            SliderBuilderOnChange::Callback(cb)
                => Self::MessageCallback(cb),
        }
    }
}

// type OnChangeCallback = Box<dyn Fn(f32) -> Message + Send + Sync>;

// pub enum SliderOnChange {
//     Message(Option<Message>),
//     Action(BuildableAction),
//     Callback(OnChangeCallback),
// }
// impl SliderOnChange {
//     pub fn resolve(
//         &self, 
//         value: f32,
//         owner: MessageOwner,
//         values: &mut dyn Reflect
//     ) -> Option<Message> {
//         match self {
//             Self::Message(m) 
//                 => m.clone(),
//             Self::Action(a) 
//                 => a.resolve(owner, values, Some(&value.into())),
//             Self::Callback(cb) 
//                 => Some((cb)(value)),
//         }
//     }
// }
// impl<T: Into<SliderOnChange>> From<Option<T>> for SliderOnChange {
//     fn from(value: Option<T>) -> Self {
//         let Some(value) = value else { return Self::Message(None) };
//         value.into()
//     }
// }
// impl From<Message> for SliderOnChange {
//     fn from(value: Message) -> Self {
//         Self::Message(Some(value))
//     }
// }
// impl From<BuildableAction> for SliderOnChange {
//     fn from(mut value: BuildableAction) -> Self {
//         if let BuildableAction::Conditional { cond, .. } = &mut value {
//             cond.build();
//         }

//         Self::Action(value)
//     }
// }
// impl From<OnChangeCallback> for SliderOnChange {
//     fn from(value: OnChangeCallback) -> Self {
//         Self::Callback(value)
//     }
// }
// impl From<SliderBuilderOnChange> for SliderOnChange {
//     fn from(value: SliderBuilderOnChange) -> Self {
//         match value {
//             SliderBuilderOnChange::Callback(cb) => Self::Callback(cb),
//             SliderBuilderOnChange::Message(m) => Self::Message(m),
//         }
//     }
// }


fn apply_snap(
    range: &RangeInclusive<f32>,
    old_value: f32,
    new_value: f32,
    snap: f32
) -> Option<f32> {
    let start = *range.start();
    let end = *range.end();
    let percent = (new_value - start) / (end - start);
    let mut new_value = (start + percent * (end - start)).clamp(start, end);
    
    // apply snap
    let diff = (old_value - new_value).abs();
    let snap_by_two = snap / 2.0;
    if diff >= snap_by_two {
        let diff2 = diff % snap;
        let sign = if new_value < 0.0 { -1.0 } else { 1.0 };

        // if the snap distance is >= half the snap amount, snap higher
        if diff2 >= snap_by_two {
            new_value += (snap - (diff % snap)) * sign;
        } else {
            new_value -= (diff % snap) * sign;
        }

        Some(new_value.clamp(start, end))
    } else {
        None
    }
}

#[test]
fn test_snap() {
    let range = -10.0..=10.0;
    let snap = 0.5f32;

    for (oldval, mut new_value, expected) in [
        (0.0, 1.0, 1.0),
        (0.0, 5.75, 6.0),
        (0.0, 0.2, 0.0),
        (0.0, 20.0, 10.0),

        (-5.0, -4.0, -4.0),
        (0.0, -5.75, -6.0),
        (0.0, -0.2, 0.0),
        (0.0, -20.0, -10.0),
    ] {
        let snapped = apply_snap(
            &range,
            oldval,
            new_value,
            snap
        );

        if let Some(snapped) = snapped {
            new_value = snapped;
        } else {
            new_value = oldval;
        }

        // check whole numbers
        println!("{new_value:.2} == {expected}");
        assert!((new_value * 100.0) as i32 == (expected * 100.0) as i32);
    }

}
