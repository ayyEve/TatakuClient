use crate::prelude::*;
use crate::prelude::ui::*;


#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("container", "text"))]
pub struct Checkbox {
    #[chain] pub style: Style,
    #[chain] pub text_style: TextStyle,
    
    pub text: String,
    pub value: CheckboxValue,

    active: bool,
    hovered: bool,

    pub on_toggle: Option<Arc<dyn Fn(bool) -> Message + Send + Sync>>,
    
    node_id: NodeId,
}
impl Checkbox {
    pub fn new(
        text: impl ToString,
        value: impl Into<CheckboxValue>,
    ) -> Self {
        Self {
            style: Style::default(),
            text_style: TextStyle {
                alignment: Alignment::CENTER_LEFT,
                .. TextStyle::default()
            },

            text: text.to_string(),
            value: value.into(),
            on_toggle: None,

            active: false,
            hovered: false,
            node_id: EMPTY_NODE,
        }
    }

    fn box_size(&self) -> Vector2 {
        Vector2::ONE * self.text_style.font_size * 0.75
    }
    fn box_padding(&self) -> Vector2 {
        Vector2::new(
            5.0,
            0.0
        )
    }

    pub fn on_toggle_arced(mut self, on_toggle: Arc<dyn Fn(bool) -> Message + Send + Sync>) -> Self {
        self.on_toggle = Some(on_toggle);
        self
    }
    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'static + Send + Sync) -> Self {
        self.on_toggle = Some(Arc::new(on_toggle));
        self
    }
}

impl Widget for Checkbox {
    fn name(&self) -> Cow<'static, str> { "checkbox_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn set_text_style(&mut self, style: TextStyle) {
        self.text_style = style;
    }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        let mut size = self.text_style.measure_text(&self.text, None);
        size += self.box_size() + self.box_padding();
        let style = Style {
            min_size: Size {
                width: Dimension::Length(size.x),
                height: Dimension::Length(size.y)
            },
            ..self.style.clone()
        };

        self.node_id = shell.tree.new_leaf(style)?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<'_>, 
    ) {
        match event.event {
            InputType::MouseMove(pos) => {
                let Some(bounds) = shell.tree.bounds(self.node_id) else { return };
                let Some(ctx) = shell.tree.get_context(self.node_id) else { return };
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
                    .map(|f| (f)(!self.value.get()))
                    ;
                if let Some(m) = m {
                    shell.publish(m);
                }

                if let CheckboxValue::Static(b) = &mut self.value {
                    *b = !*b;
                }
                // shell.event_consumed = true;
            }

            _ => {}
        }
    }

    fn draw(
        &self, 
        shell: &mut DrawShell<'_>,
    ) {
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };

        let box_size = self.box_size();
        let box_padding = self.box_padding();

        let box_bounds = Bounds::new(
            bounds.pos,
            Vector2::new(
                box_size.x + box_padding.x,
                bounds.size.y
            ),
        );

        let box_pos = Alignment::CENTER.resolve(
            &box_bounds, 
            box_size, 
            true, 
            true,
        );

        let rect = Rectangle::new(
            box_pos,
            box_size,
            if self.value.get() { shell.general_theme.active_color } else { Color::TRANSPARENT },
            Some(Border::new(shell.general_theme.get_color(self.active, self.hovered), 2.0))
        ).shape(Shape::Round(2.0));
        shell.list.push(rect);

        let text_bounds = Bounds::new(
            Vector2::new(
                bounds.pos.x + box_size.x + box_padding.x,
                bounds.pos.y
            ),
            Vector2::new(
                bounds.size.x - box_size.x,
                bounds.size.y
            )
        );

        shell.list.push(self.text_style.create_text(self.text.clone(), text_bounds))
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_> , 
        _actions: &mut ActionQueue
    ) {
        self.value.update(shell.values);
    }
}


pub enum CheckboxValue {
    Static(bool),
    Variable(BuildableCondition, bool),
}
impl CheckboxValue {
    fn get(&self) -> bool {
        match self {
            Self::Static(b) => *b,
            Self::Variable(_, b) => *b,
        }
    }
    fn update(&mut self, values: &mut dyn Reflect) {
        let Self::Variable(e, value) = self else { return };
        match e.resolve(values) {
            BuildableConditionResult::Failed => *value = false,
            BuildableConditionResult::Unbuilt(_) => unreachable!("should be built"),
            BuildableConditionResult::True => *value = true,
            BuildableConditionResult::False => *value = false,
            BuildableConditionResult::Error(shunting_yard_error) => {
                error!("{shunting_yard_error:?}");
                *value = false;
                *e = BuildableCondition::Failed;
            },
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
        Self::Variable(value, false)
    }
}
impl From<CheckboxBuilderValue> for CheckboxValue {
    fn from(value: CheckboxBuilderValue) -> Self {
        match value {
            CheckboxBuilderValue::Static(b) => Self::Static(b),
            CheckboxBuilderValue::Variable(v) => BuildableCondition::Unbuilt(v).into(),
        }
    }
}


