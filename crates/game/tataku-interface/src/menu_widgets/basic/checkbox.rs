use crate::prelude::*;

pub struct Checkbox {
    text: CheckboxText,
    value: CheckboxValue,

    active: bool,
    hovered: bool,

    on_toggle: Option<CheckboxOnToggle>,
    
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
            text,
            value: value.into(),
            on_toggle: None,

            active: false,
            hovered: false,
            node_id: EMPTY_NODE,
        }
    }

    fn box_size(&self, font_size: f32) -> Vector2 {
        Vector2::ONE * font_size * 0.75
    }
    fn box_padding(&self) -> Vector2 {
        Vector2::new(5.0, 0.0)
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

    fn size(&self, text_style: &TextStyle) -> [CssUnit; 2] {
        let text = self.text.get();
        let txt_size = text_style.measure_text(text, None);
        let box_size = self.box_size(text_style.font_size);

        let size = Vector2::new(
            box_size.x + txt_size.x,
            box_size.y.max(txt_size.y)
        ) + self.box_padding() * 2.0;

        [
            CssUnit::Pixels(f16::from_f32(size.x)),
            CssUnit::Pixels(f16::from_f32(size.y))
        ]
    }
}
impl Widget<TatakuAction> for Checkbox {
    fn name(&self) -> CowStr { "checkbox_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();
        let size = self.size(text_style);

        shell.tree.update_style(
            self.node_id, 
            |style| {
                style.min_width = CssValue::Value(size[0]);
                style.min_height = CssValue::Value(size[1]);
            }
        );
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<TatakuAction>, 
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

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        let box_size = self.box_size(text_style.font_size);
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
        ).border(Border::new(
            shell.general_theme.get_color(self.active, self.hovered), 
            2.0
        )).shape(Shape::Round(2.0));
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

        shell.list.push(text_style.create_text(
            self.text.get().to_string(), 
            text_bounds
        ));
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        self.value.update(shell.values);

        let old_text = self.text.get().to_owned();
        self.text.update(shell.values);
        let new_text = self.text.get();
        if new_text != old_text {
            let text_style = shell.tree
                .get_text_style(self.node_id)
                .unwrap();

            let size = self.size(text_style);
            shell.tree.update_style(
                self.node_id, 
                |style| {
                    style.min_width = size[0].into();
                    style.min_height = size[1].into();
                }
            );
        }
    }
}


#[derive(Debug)]
pub enum CheckboxText {
    Static(ArcStr),
    Variable(BuildableText, String),
    Buildable(BuildableText, String),
}
impl CheckboxText {
    fn build(&mut self) -> Result<(), BuildableShuntingYardError> {
        match self {
            Self::Buildable(b, _) => b.compute(),
            Self::Variable(b, _) => b.compute(),
            _ => Ok(())
        }
    }

    fn get(&self) -> &str {
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
        Self::Static(value.into())
    }
}
impl From<String> for CheckboxText {
    fn from(value: String) -> Self {
        Self::Static(value.into())
    }
}
impl From<ArcStr> for CheckboxText {
    fn from(value: ArcStr) -> Self {
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
    Variable{
        path: VariablePathResolver, 
        cache: bool, 
        failed: bool,
    },
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
    Buildable(Box<BuildableAction>),
}
impl CheckboxOnToggle {
    fn run(
        &self, 
        value: bool, 
        node: NodeId, 
        values: &mut dyn Reflect,
    ) -> Option<Result<Message, TatakuAction>> {
        match self {
            Self::Callback(cb) => Some(Ok(cb(value))),

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
        Self::Buildable(Box::new(value))
    }
}
