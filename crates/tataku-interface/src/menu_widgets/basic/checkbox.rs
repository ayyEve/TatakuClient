use crate::prelude::*;
use crate::prelude::ui::*;


#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("container", "text"))]
pub struct Checkbox {
    #[chain] pub style: Style,
    #[chain] pub text_style: TextStyle,
    
    pub text: CheckboxText,
    pub value: CheckboxValue,

    active: bool,
    hovered: bool,

    pub on_toggle: Option<CheckboxOnToggle>,
    
    node_id: NodeId,
}
impl Checkbox {
    pub fn new(
        text: impl Into<CheckboxText>,
        value: impl Into<CheckboxValue>,
    ) -> Self {
        let mut text = text.into();
        if let Err(e) = text.build() {
            warn!("error building text: {e:?}");
        }

        Self {
            style: Style::default(),
            text_style: TextStyle {
                alignment: Alignment::CENTER_LEFT,
                .. TextStyle::default()
            },

            text,
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


    fn size(&self) -> Size<Dimension> {
        let text = self.text.get();
        let mut size = self.text_style.measure_text(text, None);
        size += self.box_size() + self.box_padding();
        Size {
            width: Dimension::Length(size.x),
            height: Dimension::Length(size.y)
        }
    }
}
impl Widget for Checkbox {
    fn name(&self) -> Cow<'static, str> { "checkbox_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn set_text_style(&mut self, style: TextStyle) {
        self.text_style = style;
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        let style = Style {
            min_size: self.size(),
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
        shell: &mut InputShell, 
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
                    .and_then(|f| f.run(
                        !self.value.get(), 
                        self.node_id, 
                        shell.values
                    ))
                    ;

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

    fn draw(&self, shell: &mut DrawShell) {
        let Some(bounds) = shell.tree.absolute_bounds(self) 
        else { return };

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
            if self.value.get() { 
                shell.general_theme.active_color 
            } else { 
                Color::TRANSPARENT 
            }
        )
            .border(Border::new(
                shell.general_theme.get_color(self.active, self.hovered), 
                2.0
            ))
            .shape(Shape::Round(2.0));
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

        shell.list.push(self.text_style.create_text(
            self.text.get().clone(), 
            text_bounds
        ));
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.value.update(shell.values);

        let old_text = self.text.get().clone();
        self.text.update(shell.values);
        let new_text = self.text.get();
        if new_text != &old_text {
            let Some(ctx) = shell.tree.get_context(self.node_id) 
            else { return };

            self.text_style = ctx.element_data.style().0.text_style(shell.values);

            let size = self.size();
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::UpdateStyleWith(Box::new(move |style| {
                    style.min_size = size;
                }))
            ));
        }
    }
}


#[derive(Debug)]
pub enum CheckboxText {
    Static(String),
    Variable(BuildableText, String),
    Buildable(BuildableText, String),
}
impl CheckboxText {
    fn build(&mut self) -> Result<(), ShuntingYardError> {
        match self {
            Self::Buildable(b, _) => b.compute(),
            Self::Variable(b, _) => b.compute(),
            _ => Ok(())
        }
    }

    fn get(&self) -> &String {
        match self {
            Self::Static(t) => t,
            Self::Variable(_, t) => t,
            Self::Buildable(_, t) => t,
        }
    }
    fn update(&mut self, values: &dyn Reflect) {
        match self {
            Self::Static(_) => {},
            Self::Variable(path, cache) => {
                let path = path.to_string(values);
                if let Ok(value) = values.reflect_display(&path, None) {
                    *cache = value;
                } else {
                    *cache = format!("failed: {path}");
                }
            }
            Self::Buildable(
                b, 
                cache
            ) => *cache = b.to_string(values),
        }
    }
}
impl From<&str> for CheckboxText {
    fn from(value: &str) -> Self {
        Self::Static(value.to_owned())
    }
}
impl From<String> for CheckboxText {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}
impl From<BuildableText> for CheckboxText {
    fn from(value: BuildableText) -> Self {
        Self::Buildable(value, String::new())
    }
}

#[derive(Debug)]
pub enum CheckboxValue {
    Static(bool),
    Variable(String, bool, bool),
    Condition(BuildableCondition, bool),
}
impl CheckboxValue {
    pub fn condition(value: impl Into<BuildableCondition>) -> Self {
        let value = value.into();
        Self::Condition(value, false)
    }

    fn get(&self) -> bool {
        match self {
            Self::Static(b) => *b,
            Self::Variable(_, b, _) => *b,
            Self::Condition(_, b) => *b,
        }
    }
    fn update(&mut self, values: &mut dyn Reflect) {
        match self {
            Self::Static(_) => {},
            Self::Variable(path, value, failed) => {
                match values.reflect_get::<bool>(&*path) {
                    Ok(val) => *value = val.copied(),
                    Err(e) => if !*failed {
                        *failed = true;
                        error!("error with checkbox variable: {e:?}");
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
impl From<CheckboxBuilderValue> for CheckboxValue {
    fn from(value: CheckboxBuilderValue) -> Self {
        match value {
            CheckboxBuilderValue::Static(b) => Self::Static(b),
            CheckboxBuilderValue::Variable(v) 
                => BuildableCondition::Unbuilt(v).into(),
        }
    }
}


#[derive(Debug2)]
pub enum CheckboxOnToggle {
    #[debug(skip)]
    Callback(Arc<dyn Fn(bool) -> Message + Send + Sync>),
    Buildable(BuildableAction),
}
impl CheckboxOnToggle {
    fn run(
        &self, 
        value: bool, 
        node: NodeId, 
        values: &mut dyn Reflect,
    ) -> Option<Result<Message, TatakuAction>> {
        match self {
            Self::Callback(cb) 
                => Some(Ok(cb(value))),

            Self::Buildable(action) => {
                let mut action = action.clone();
                action.build(values);
                
                action.into_action(
                    node, 
                    values, 
                    Some(&TatakuValue::Bool(value))
                ).map(Err)
            },
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
        Self::Buildable(value)
    }
}
