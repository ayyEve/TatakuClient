use crate::prelude::*;
use common::reflect::*;
use tataku::{
    Color,
    Border,
    Vector2,
    TatakuValue,
};
use ui::{
    tree::*,
    style::*,
    widget::*,
    message::*,
};
use input::{ 
    InputType,
    InputEvent, 
    MouseButton, 
};

const BOX_SIZE_EM: f32 = 0.75;

pub struct Checkbox {
    value: CheckboxValue,

    active: bool,
    hovered: bool,

    on_toggle: Option<CheckboxOnToggle>,
    
    node_id: NodeId,
}
impl Checkbox {
    pub fn new(value: CheckboxValue) -> Self {
        Self {
            value,
            on_toggle: None,

            active: false,
            hovered: false,
            node_id: ui::EMPTY_NODE,
        }
    }

    pub fn on_toggle_arced(mut self, on_toggle: Arc<dyn Fn(bool) -> Message + Send + Sync>) -> Self {
        self.on_toggle = Some(on_toggle.into());
        self
    }
    pub fn on_toggle(self, on_toggle: impl Fn(bool) -> Message + 'static + Send + Sync) -> Self {
        self.on_toggle_arced(Arc::new(on_toggle))
    }

    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Into<CheckboxOnToggle>>) -> Self {
        if let Some(toggle) = on_toggle {
            self.on_toggle = Some(toggle.into());
        }
        self
    }
}
impl Widget<actions::Action> for Checkbox {
    fn name(&self) -> CowStr { "checkbox_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        shell.tree.update_style(
            self.node_id,
            |style| {
                style.min_width = CssValue::Value(CssUnit::Em(f16::from_f32(BOX_SIZE_EM)));
                style.min_height = CssValue::Value(CssUnit::Em(f16::from_f32(BOX_SIZE_EM)));
            }
        );
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<actions::Action>, 
    ) {
        match event.event {
            InputType::MouseMove(pos) => {
                let Some(bounds) = shell.tree.bounds(self.node_id) 
                else { return };

                let Some(ctx) = shell.tree.get_context(self.node_id) 
                else { return };

                let pos = ctx.inverse_global_transform * pos;

                self.hovered = bounds.contains(pos);
                if self.active { self.active = false }
            }

            InputType::MousePress(MouseButton::Left) if self.hovered => {
                self.active = true;
                shell.event_consumed = true;
            }

            InputType::MouseRelease(MouseButton::Left) if self.active => {
                let m = self.on_toggle
                    .as_ref()
                    .map(|f| f.run(
                        !self.value.get(), 
                        self.node_id, 
                        shell.source,
                        shell.values
                    ));

                if let Some(m) = m {
                    match m {
                        Ok(m) => shell.publish(m),
                        Err(action) => shell.actions.push(action),
                    }
                }

                if let CheckboxValue::Static(b) = &mut self.value {
                    *b = !*b;
                }
                // shell.event_consumed = true;
            }

            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        let size = BOX_SIZE_EM * text_style.font_size;

        shell.list.push(
            graphics::Rectangle::new(
                Vector2::ONE * size,
                if self.value.get() {
                    shell.general_theme.active_color
                } else {
                    Color::TRANSPARENT
                }
            ).border(Border::new(
                shell.general_theme.get_color(self.active, self.hovered),
                2.0
            )).shape(graphics::Shape::Round(2.0))
            .with_transform(tataku::Matrix::identity()
                .trans(bounds.pos)
            )
        );
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        self.value.update(shell.values);
    }
}

#[derive(Debug)]
pub enum CheckboxValue {
    Static(bool),
    Variable {
        path: engine::VariablePathResolver, 
        cache: bool, 
        failed: bool,
    },
    Condition(BuildableCondition, bool),
}
impl CheckboxValue {
    fn get(&self) -> bool {
        match self {
            Self::Static(b) => *b,
            Self::Variable { cache: b, .. } => *b,
            Self::Condition(_, b) => *b,
        }
    }
    fn update(&mut self, values: &mut dyn Reflect) {
        match self {
            Self::Static(_) => {},
            Self::Variable { 
                path, 
                cache, 
                failed
             } => {
                let Ok(path) = path
                .resolve_path(values)
                .map_err(|e| {
                    *failed = true;
                    error!("Error with checkbox path: {e:?}");
                }) else { return };

                match values.reflect_get::<bool>(&*path) {
                    Ok(val) => *cache = val.copied(),
                    Err(e) => if !*failed {
                        *failed = true;
                        error!("Error with checkbox variable: {e:?}");
                    }
                }
            }
            Self::Condition(e, value) => {
                if e.is_unbuilt() {
                    e.build();
                }

                match e.resolve(values) {
                    BuildableConditionResult::Failed => *value = false,
                    BuildableConditionResult::Unbuilt(_) => unreachable!("should be built"),
                    BuildableConditionResult::True => *value = true,
                    BuildableConditionResult::False => *value = false,
                    BuildableConditionResult::Error(shunting_yard_error) => {
                        error!("{shunting_yard_error:?}");
                        *value = false;
                        *e = BuildableCondition::Failed;
                    }
                }
            }
        }
    }
}
impl From<bool> for CheckboxValue {
    fn from(value: bool) -> Self {
        Self::Static(value)
    }
}
impl From<BuildableCondition> for CheckboxValue {
    fn from(mut value: BuildableCondition) -> Self {
        value.build();
        Self::Condition(value, false)
    }
}


#[derive(Debug2)]
pub enum CheckboxOnToggle {
    #[debug(skip)]
    Callback(Arc<dyn Fn(bool) -> Message + Send + Sync>),
    Buildable(Vec<BuildableAction>),
}
impl CheckboxOnToggle {
    fn run(
        &self, 
        value: bool, 
        node: NodeId, 
        source: ui::MessageSource,
        values: &mut dyn Reflect,
    ) -> Result<Message, actions::Action> {
        match self {
            Self::Callback(cb) => Ok(cb(value)),

            Self::Buildable(actions) => {
                let passed_in = Some(&TatakuValue::Bool(value));

                // todo: error on failed
                let actions = actions.iter()
                    .cloned()
                    .filter_map(|action| action.resolve(node, source, values, passed_in))
                    .collect();

                Err(actions::Action::Multiple(actions))
            },
        }
    }

    pub fn from_buildables(mut values: Vec<BuildableAction>) -> Option<Self> {
        for value in values.iter_mut() {
            if let BuildableAction::Conditional {
                cond,
                ..
            } = value {
                cond.build();
            }
        }

        if values.is_empty() {
            None
        } else {
            Some(Self::Buildable(values))
        }
    }
}
impl From<Arc<dyn Fn(bool) -> Message + Send + Sync>> for CheckboxOnToggle {
    fn from(value: Arc<dyn Fn(bool) -> Message + Send + Sync>) -> Self {
        Self::Callback(value)
    }
}
impl From<BuildableAction> for CheckboxOnToggle {
    fn from(value: BuildableAction) -> Self {
        Self::from_buildables(vec![value]).unwrap()
    }
}
