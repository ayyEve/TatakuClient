use crate::prelude::*;
use std::ops::RangeInclusive;


// TODO: should we make this generic? maybe use a ReflectNumber instead of f32?
#[derive(ChainableInitializer)]
pub struct Slider {
    value: SliderValue,
    min: SliderValue,
    max: SliderValue,

    #[chain] step: Option<SliderValue>,
    on_change: Option<InputAction<f32>>, 

    hovered: bool,
    pressed: bool,
    node_id: NodeId,
}
impl Slider {
    pub fn new(
        min: impl Into<SliderValue>,
        max: impl Into<SliderValue>,
        value: impl Into<SliderValue>,
        on_change: Option<impl Into<InputAction<f32>>>,
    ) -> Self {
        Self {
            min: min.into(),
            max: max.into(),
            // range,
            value: value.into(),
            step: None,
            on_change: on_change.map(|i| i.into()),
            
            hovered: false,
            pressed: false,
            node_id: EMPTY_NODE,
        }
    }

    fn range(&self) -> RangeInclusive<f32> {
        self.min.get()..=self.max.get()
    }
}
impl Widget<TatakuAction> for Slider {
    fn name(&self) -> CowStr { "slider_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        // let style = CssStyle {
        //     min_width: CssUnit::Pixels(half::f16::from_f32(100.0)).into(),
        //     min_height: CssUnit::Pixels(half::f16::from_f32(30.0)).into(),
        //     ..self.style.clone()
        // };

        self.node_id = shell.tree.new_leaf()?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        shell.tree.update_style(
            self.node_id, 
            |style| {
                style.min_width = CssUnit::Pixels(half::f16::from_f32(100.0)).into();
                style.min_height = CssUnit::Pixels(half::f16::from_f32(30.0)).into();
            }
        );
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<TatakuAction>,
    ) {
        if let Some(on_change) = self.on_change.as_mut()
        && !on_change.is_built() {
            on_change.build(shell.values);
        }


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
                    let range = self.range();
                    let start = *range.start();
                    let end = *range.end();

                    let percent = (pos.x - bounds.pos.x) / bounds.size.x;
                    let mut new_value = f32::lerp(start, end, percent)
                        .clamp(start, end);
                    // (start + percent * (end - start))
                    //     .clamp(start, end);

                    if let Some(snap) = &self.step {
                        // apply_snap
                        if let Some(val) = apply_snap(
                            &range, 
                            value, 
                            new_value, 
                            snap.get()
                        ) { 
                            new_value = val;
                        } else {
                            new_value = value;
                        }
                    }

                    if (value - new_value).abs() > f32::EPSILON {
                        self.value.set(new_value);
                        if let Some(on_change) = &self.on_change {
                            on_change.run(
                                &new_value,
                                self.node_id,
                                shell.messages,
                                shell.actions,
                                shell.values,
                            );
                        } else if let SliderValue::Variable { 
                            variable, .. 
                        } = &self.value {
                            let Ok(path) = variable
                                .resolve_path(shell.values) 
                            else { return };

                            let _ = shell.values.reflect_insert(
                                &*path, 
                                new_value
                            );
                        }
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
                let range = self.range();

                match key {
                    Key::Left => if active || self.hovered {
                        shell.event_consumed = true;
                        self.value.set((
                            self.value.get() - self.step
                                .as_ref()
                                .map_or(1.0, |s| s.get())
                            )
                            .clamp(*range.start(), *range.end())
                        );
                    }
                    Key::Right => if active || self.hovered {
                        shell.event_consumed = true;
                        self.value.set((
                            self.value.get() + self.step
                                .as_ref()
                                .map_or(1.0, |s| s.get())
                            )
                            .clamp(*range.start(), *range.end())
                        );
                    }

                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        let _ = self.value.update(shell.values);
        let _ = self.min.update(shell.values);
        let _ = self.max.update(shell.values);
        if let Some(s) = self.step.as_mut() { 
            let _ = s.update(shell.values);
        }
    }

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
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
        let start = self.min.get();
        let end = self.max.get();
        let percent = (self.value.get() - start) / (end - start);

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
        )));
    }
}

pub enum SliderValue {
    Static(f32),
    Variable {
        variable: VariablePathResolver,
        value: f32,
    },
    Buildable {
        buildable: BuildableValue,
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
            Self::Buildable { value, .. } => *value,
        }
    }
    fn set(&mut self, new: f32) {
        match self {
            Self::Error => {},
            Self::Static(v) => *v = new,
            Self::Variable { value, .. } => *value = new,
            Self::Buildable { value, .. } => *value = new,
        }
    }

    fn update(&mut self, values: &dyn Reflect) -> TatakuResult<()> {
        match self {
            Self::Static(_) | Self::Error => {},
            Self::Variable {
                variable,
                value
            } => {
                let path = variable
                    .resolve_path(values)?;

                match values.reflect_as_number(&*path) {
                    Ok(n) => *value = n.into(),
                    Err(e) => {
                        warn!("error getting value at path '{path}': {e:?}");
                        *self = Self::Error;
                    }
                }
            }

            Self::Buildable { 
                buildable, 
                value 
            } => if let Some(t) = buildable
                .resolve(values, None) 
            && let Some(v) = t.as_f32() {
                *value = v;
            }
        }

        Ok(())
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
            variable: variable.into(),
            value: 0.0,
        }
    }
}
impl From<VariablePathResolver> for SliderValue {
    fn from(variable: VariablePathResolver) -> Self {
        Self::Variable {
            variable,
            value: 0.0,
        }
    }
}
impl From<BuildableValue> for SliderValue {
    fn from(value: BuildableValue) -> Self {
        match value {
            BuildableValue::Variable(variable) => Self::Variable {
                variable,
                value: 0.0
            },

            buildable => Self::Buildable { 
                buildable, 
                value: 0.0
            }
        }
    }
}




fn apply_snap(
    range: &RangeInclusive<f32>,
    old_value: f32,
    new_value: f32,
    snap: f32
) -> Option<f32> {
    let start = *range.start();
    let end = *range.end();
    let percent = (new_value - start) / (end - start);
    let mut new_value = f32::lerp(start, end, percent).clamp(start, end);
    
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
