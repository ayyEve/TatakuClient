use crate::prelude::*;
use common::reflect::*;
use widgets::InputAction;
use std::ops::RangeInclusive;
use tataku_engine_common::math::Interpolation;

use tataku::{
    Color,
    Bounds,
    Border,
    Vector2,
};
use ui::{
    tree::*,
    style::*,
    widget::*,
};
use input::{
    Key,
    InputType,
    InputEvent,
    MouseButton,
};


// TODO: should we make this generic? maybe use a ReflectNumber instead of f32?
#[derive(ChainableInitializer)]
pub struct Slider {
    value: SliderValue,
    min: SliderValue,
    max: SliderValue,

    #[chain] step: Option<SliderValue>,
    on_change: InputAction<f32>,

    node_id: NodeId,
}
impl Slider {
    pub fn new(
        min: SliderValue,
        max: SliderValue,
        value: SliderValue,
        on_change: InputAction<f32>,
    ) -> Self {
        Self {
            min,
            max,
            // range,
            value,
            step: None,
            on_change,

            node_id: ui::EMPTY_NODE,
        }
    }

    fn range(&self) -> RangeInclusive<f32> {
        self.min.get()..=self.max.get()
    }
}
impl Widget<actions::Action> for Slider {
    fn name(&self) -> CowStr { "slider_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<actions::Action>
    ) -> taffy::TaffyResult<NodeId> {
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

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        shell.tree.set_overrides(
            self.node_id,
            |style| {
                style.min_width = Some(CssUnit::Pixels(f16::from_f32(100.0)));
                style.min_height = Some(CssUnit::Pixels(f16::from_f32(30.0)));
            }
        );
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        let Some(content_bounds) = shell.tree.content_bounds(self.node_id)
        else { return };
        let Some(ctx) = shell.tree.get_context_mut(self.node_id)
        else { return };
        let state = &mut ctx.element_data.state;
        let focused = ctx.selected == Some(true) || state.focus();

        match &event.event {
            InputType::MouseMove(pos) => {
                let pos = ctx.inverse_global_transform * *pos;
                state.set_hover(content_bounds.contains(pos));

                if state.active() {
                    let value = self.value.get();
                    let range = self.range();
                    let start = *range.start();
                    let end = *range.end();

                    let percent = (pos.x - content_bounds.pos.x) / content_bounds.size.x;
                    let mut new_value = f32::lerp(start, end, percent)
                        .clamp(start, end);
                    // (start + percent * (end - start))
                    //     .clamp(start, end);

                    if let Some(snap) = &self.step {
                        // apply_snap
                        new_value = apply_snap(
                            &range,
                            new_value,
                            snap.get()
                        );
                    }

                    if (value - new_value).abs() > f32::EPSILON {
                        self.value.set(new_value);

                        self.on_change.run(
                            &new_value,
                            self.node_id,
                            shell.source,
                            shell.messages,
                            shell.actions,
                            shell.values,
                        );

                        if let SliderValue::Variable {
                            variable, ..
                        } = &self.value {
                            let Ok(path) = variable.resolve_path(shell.values)
                            else { return };

                            let _ = shell.values.reflect_insert(
                                &*path,
                                new_value
                            );
                        }
                    }
                }
            }

            InputType::MousePress(MouseButton::Left) if state.hover() => {
                state.set_active(true);
                shell.event_consumed = true;
            }
            InputType::MouseRelease(MouseButton::Left) => {
                state.set_active(false);
                // never consume a mouse release event
            }

            InputType::MousePressCancel(MouseButton::Left) => {
                state.set_active(false);
                shell.event_consumed = true;
            }

            InputType::KeyPress(press) => {
                let Some(key) = press.key else { return };
                let range = self.range();

                match key {
                    Key::Left => if focused || state.hover() {
                        shell.event_consumed = true;
                        self.value.set((
                            self.value.get() - self.step
                                .as_ref()
                                .map_or(1.0, |s| s.get())
                            )
                            .clamp(*range.start(), *range.end())
                        );
                    }
                    Key::Right => if focused || state.hover() {
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

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        if let Err(e) = self.value.update(shell.values) {
            println!("fuck: {e:?}");
        }
        let _ = self.min.update(shell.values);
        let _ = self.max.update(shell.values);
        if let Some(s) = self.step.as_mut() {
            let _ = s.update(shell.values);
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some((bounds, state)) = shell.with_ctx(
            self.node_id, 
            |c| (c.absolute_bounds, c.element_data.state)
        ) else { return };
        

        shell.list.push(
            graphics::Rectangle::new_bounds(bounds, Color::TRANSPARENT)
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

        shell.list.push(graphics::Rectangle::new_bounds(bounds, Color::BLACK));

        // draw slider
        let start = self.min.get();
        let end = self.max.get();
        let percent = (self.value.get() - start) / (end - start);

        let dragger_pos = Vector2::new(
            bounds.pos.x + bounds.size.x * percent.clamp(0.0, 1.0),
            track.pos.y
        );

        shell.list.push(graphics::Circle::new(
            dragger_pos,
            (bounds.size.y / 2.0) * 5.0/6.0,
            shell.general_theme.default_color
        ).border(Border::new(
            shell.general_theme.for_state(state),
            2.0
        )));
    }
}

pub enum SliderValue {
    Static(f32),
    Variable {
        variable: engine::VariablePathResolver,
        value: f32,
        error_printed: bool,
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

    fn update(&mut self, values: &dyn Reflect) -> tataku::Result<()> {
        match self {
            Self::Static(_) | Self::Error => {},
            Self::Variable {
                variable,
                value,
                error_printed,
            } => {
                let path = variable
                    .resolve_path(values)?;

                match values.reflect_as_number(&*path) {
                    Ok(n) => {
                        if *error_printed { *error_printed = false; }
                        *value = n.into();
                    },
                    Err(e) => {
                        if !*error_printed {
                            warn!("error getting value at path '{path}': {e:?}");
                            *error_printed = true;
                        }
                    }
                }
            }

            Self::Buildable {
                buildable,
                value,
            } => if let Some(t) = buildable
                    .resolve(values, None)
                && let Some(v) = t.as_f32()
            {
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
            error_printed: false,
        }
    }
}
impl From<engine::VariablePathResolver> for SliderValue {
    fn from(variable: engine::VariablePathResolver) -> Self {
        Self::Variable {
            variable,
            value: 0.0,
            error_printed: false,
        }
    }
}
impl From<BuildableValue> for SliderValue {
    fn from(mut value: BuildableValue) -> Self {
        value.build();

        match value {
            BuildableValue::Variable(variable) => Self::Variable {
                variable,
                value: 0.0,
                error_printed: false,
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
    new_value: f32,
    snap: f32
) -> f32 {
    let start = *range.start();
    let end = *range.end();

    let offset = new_value - start;
    let snapped_offset = (offset / snap).round() * snap;

    let percent = snapped_offset / (end - start);

    f32::lerp(start, end, percent).clamp(start, end)
}

#[test]
fn test_snap() {
    let range = -10.0..=10.0;
    let snap = 0.5f32;

    for (_oldval, mut new_value, expected) in [
        (0.0, 1.0, 1.0),
        (0.0, 5.75, 6.0),
        (0.0, 0.2, 0.0),
        (0.0, 20.0, 10.0),

        (-5.0, -4.0, -4.0),
        (0.0, -5.75, -6.0),
        (0.0, -0.2, 0.0),
        (0.0, -20.0, -10.0),
    ] {
        new_value = apply_snap(
            &range,
            new_value,
            snap
        );

        // check whole numbers
        println!("{new_value:.2} == {expected}");
        assert!((new_value * 100.0) as i32 == (expected * 100.0) as i32);
    }

}
