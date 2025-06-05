use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("text", "container"))]
pub struct Dropdown {
    #[chain] pub style: Style,
    pub text_style: TextStyle,

    pub placeholder: String,
    pub value: DropdownValue,
    pub variants: DropdownVariants,

    pub on_change: DropdownOnChange,
    // pub theme: DropdownTheme,

    /// is dropdown visible?
    active: bool,
    active_index: Option<usize>,

    hover: bool,

    node_id: NodeId,
}
impl Dropdown {
    pub fn new(
        variants: impl Into<DropdownVariants>,
        value: impl Into<DropdownValue>,
        on_change: impl Into<DropdownOnChange>,
    ) -> Self {
        let variants = variants.into();
        let value = value.into();

        Self {
            style: Style::default(),
            text_style: TextStyle {
                alignment: Alignment::CENTER,
                ..Default::default()
            },
            value,

            placeholder: String::new(),
            variants,
            on_change: on_change.into(),
            active: false,
            hover: false,
            active_index: None,

            // theme: DropdownTheme::sane_defaults(),

            node_id: EMPTY_NODE
        }
    }

    pub fn placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    fn get_style(&self, scale: Option<Vector2>) -> Style {
        let placeholder_size = self
            .text_style
            .measure_text(&self.placeholder, scale);
        
        let largest_text = self.variants.get_displays()
            .iter()
            .map(|a| self.text_style.measure_text(a, scale))
            .fold(
                placeholder_size, 
                |a, b| Vector2::new(a.x.max(b.x), a.y.max(b.y))
            )
            ;
        
        Style {
            min_size: Size {
                width: Dimension::Length(largest_text.x),
                height: Dimension::Length(largest_text.y)
            },
            // padding: Padding::from(ElementPadding::Single(5.0)).0,

            ..self.style.clone()
        }
    }


    fn set_value(
        &mut self, 
        index: usize, 
        shell: &mut InputShell
    ) {
        self.active = false;
        self.value.set_index(index);
        debug!("setting value to {index} ({})", self.variants.get_displays()[index]);

        let message = match &self.on_change {
            DropdownOnChange::Message(message) => message.clone(),
            DropdownOnChange::Buildable(buildable_action) => {
                let passed_in = match &self.variants {
                    DropdownVariants::Static(items) 
                        => Some(items[index].clone().into()),
                    DropdownVariants::Variable(_) => None,
                    DropdownVariants::Built { 
                        items, 
                        .. 
                    } => {
                        TatakuValue::from_reflection(
                            items[index].value.duplicate().unwrap()
                        )
                            .inspect_err(|e| warn!("didnt reflect: {e:?}"))
                            .ok()
                    },
                };



                let action = buildable_action.clone().into_action(
                    self.node_id, 
                    shell.values, 
                    passed_in.as_ref(),
                );
                if let Some(action) = action {
                    shell.actions.push(action);
                }

                None
            },
            DropdownOnChange::Callback(f) => Some(f(index)),
        };

        if let Some(m) = message {
            shell.messages.push(m);
        }
    }
}
impl Widget for Dropdown {
    fn name(&self) -> Cow<'static, str> { "dropdown_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }


    fn update_styles(
        &mut self, 
        shell: &mut StyleShell,
        _display_override: Option<ui::Display>
    ) {
        let text_style = shell
            .tree
            .get_context(self.node_id).unwrap()
            .element_data.style()
            .0.text_style(shell.values);
        
        let placeholder_size = text_style.measure_text(&self.placeholder, None);
        
        let largest_text = self.variants.get_displays()
            .iter()
            .map(|a| text_style.measure_text(a, None))
            .fold(placeholder_size, |a, b| Vector2::new(a.x.max(b.x), a.y.max(b.y)))
            ;

        let mut style = shell.tree.get_style(self.node_id).unwrap().clone();
        style.min_size = Size {
            width: Dimension::Length(largest_text.x),
            height: Dimension::Length(largest_text.y),
        };
        shell.tree.set_style(self.node_id, style);
    }

    fn set_text_style(&mut self, style: TextStyle) {
        self.text_style = style;
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        let style = self.get_style(Some(Vector2::ONE * shell.ui_scale));
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
        let Some(bounds) = shell.tree.bounds(&*self) else { return };
        let Some(context) = shell.tree.get_context(&*self) else { return };

        match &event.event {
            InputType::KeyPress(input) if self.active => {
                let Some(key) = input.as_key() else { return };
                
                match key {
                    Key::Escape => {
                        self.active = false;
                        self.active_index = None;
                    }
                    Key::Enter => {
                        if let Some(index) = self.active_index.take() {
                            self.set_value(index, shell);
                        }
                    }

                    _ => {}
                }
            }

            InputType::MouseMove(pos) => {
                let pos = context.inverse_global_transform * *pos;
                self.hover = bounds.contains(pos);

                if self.active {
                    let bounds_with_dropdown = Bounds::new(
                        bounds.pos,
                        Vector2::new(
                            bounds.size.x,
                            bounds.size.y * (self.variants.len() + 1) as f32,
                        )
                    );

                    if !bounds_with_dropdown.contains(pos) { return }

                    // at this point we know something is hovered over, find out what
                    let rel_y = pos.y - bounds.pos.y;
                    let mut index = (rel_y / bounds.size.y) as usize;
                    
                    if index == 0 { return } // 0 would be the dropdown itself, not an item in the list
                    index -= 1;

                    self.active_index = Some(index);
                }
            }
            InputType::MousePress(MouseButton::Left) if self.active => {
                let pos = context.inverse_global_transform * event.mouse_pos;

                let bounds_with_dropdown = Bounds::new(
                    bounds.pos,
                    Vector2::new(
                        bounds.size.x,
                        bounds.size.y * (self.variants.len() + 1) as f32,
                    )
                );

                if !bounds_with_dropdown.contains(pos) {
                    self.active = false;
                    self.active_index = None;
                    // shell.event_consumed = true;
                    return
                }

                if let Some(index) = self.active_index.take() {
                    self.set_value(index, shell);
                } else {
                    self.active = false;
                }
                shell.event_consumed = true;
            }
            InputType::MousePress(MouseButton::Left) if !self.active => {
                // let pos = context.inverse_global_transform * event.mouse_pos;
                self.active = self.hover;
                if self.active {
                    shell.event_consumed = true;
                }
            }

            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell) {
        let theme = &shell.general_theme;
        let Some(bounds) = shell.tree.absolute_bounds(self) 
        else { return };

        // bounding box
        shell.list.push(
            Rectangle::new_bounds(
                bounds,
                theme.background_color,
            )
            .border(Border::new(theme.get_color(self.active, self.hover), 2.0))
        );

        // selected text
        let displays = self.variants.get_displays();
        let main_text = self.value.index()
            .and_then(|n| displays.get(n))
            .unwrap_or(&self.placeholder);

        shell.list.push(self.text_style.create_text(main_text.clone(), bounds));
    }

    fn draw_overlay(&self, shell: &mut DrawShell) {
        if !self.active { return }
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };
        let theme = &shell.general_theme;

        let selected = self.value.index().unwrap_or(self.variants.len());
        let active = self.active_index.unwrap_or(self.variants.len());

        // draw all options
        // TODO: margin between items
        for (n, i) in self.variants.get_displays().iter().cloned().enumerate() {
            let offset = Vector2::new(
                bounds.pos.x,
                bounds.pos.y + bounds.size.y * (n + 1) as f32,
            );

            // bounding box
            shell.list.push(
                Rectangle::new(
                    offset, 
                    bounds.size,
                    theme.background_color.alpha(1.0),
                )
                .border(Border::new(
                    theme.get_color(n == selected, n == active), 
                    2.0
                ))
            );

            let text = self.text_style.create_text(i, Bounds::new(offset, bounds.size));
            shell.list.push(text);
        }
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        if self.variants.is_unbuilt() {
            if let Err(e) = self.variants.build(shell.values) {
                error!("error building variants: {e:?}");
                return 
            }

            shell.tree.set_style(self.node_id, self.get_style(None));
        }

        if let DropdownValue::Variable(var, index) = &mut self.value {
            let selected = match shell.values.impl_get(ReflectPath::new(&*var)) {
                Ok(s) => match TatakuValue::from_reflection(s)
                    .map(|s| s.as_string()) 
                {
                    Ok(s) => Some(s),
                    Err(ReflectError::OptionIsNone) => None,
                    Err(e) => {
                        error!("dropdown error: {e:?}");
                        return 
                    }
                }
                Err(ReflectError::EntryNotExist { .. }) => None,
                Err(e) => {
                    error!("dropdown error: {e:?}");
                    return
                }
            };
        
            if selected.is_none() && index.is_some() {
                *index = None;
                return;
            }


            if let Some(selected) = selected {
                if index.is_none() {
                    if let DropdownVariants::Static(list) = &self.variants {
                        if let Some((n, _)) = list
                            .iter()
                            .enumerate()
                            .find(|(_, a)| *a == &selected) 
                        {
                            *index = Some(n);
                        }
                    }
                }


                let DropdownVariants::Built { items, .. } = &self.variants 
                else { return };

                for (n, i) in items.iter().enumerate() {
                    if i.id == selected {
                        *index = Some(n);
                        return;
                    }
                }
            }

            *index = None;
        }

    }
}


type OnChange = Box<dyn Fn(usize) -> Message + Send + Sync>;
pub enum DropdownOnChange {
    Message(Option<Message>),
    Buildable(BuildableAction),
    Callback(OnChange),
}
impl <T: Into<DropdownOnChange>> From<Option<T>> for DropdownOnChange {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl From<Message> for DropdownOnChange {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl From<OnChange> for DropdownOnChange {
    fn from(value: OnChange) -> Self {
        Self::Callback(value)
    }
}
impl From<BuildableAction> for DropdownOnChange {
    fn from(mut value: BuildableAction) -> Self {
        if let BuildableAction::Conditional { cond, .. } = &mut value {
            cond.build();
        }

        Self::Buildable(value)
    }
}
impl From<DropdownBuilderOnChange> for DropdownOnChange {
    fn from(value: DropdownBuilderOnChange) -> Self {
        match value {
            DropdownBuilderOnChange::Message(message) => Self::Message(message),
            DropdownBuilderOnChange::Callback(cb) => Self::Callback(cb),
        }
    }
}

pub enum DropdownVariants {
    Static(Vec<String>),
    Variable(String),
    Built {
        items: Vec<DropdownWrapper>,
        cached_displays: Vec<String>
    },
}
impl DropdownVariants {
    fn is_unbuilt(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
    
    fn build(&mut self, values: &dyn Reflect) -> TatakuResult<()> {
        let Self::Variable(var) = self else { return Ok(()) };

        let iter = values.reflect_iter(&*var)?;
        let items = iter.filter_map(|value| {
            let id = TatakuValue::from_reflection(value.item).ok()?.as_string();
            Some(DropdownWrapper {
                display: value.impl_display(ReflectPath::new(""), None).unwrap_or_else(|_| id.clone()), 
                id,
                value: value.duplicate().expect("Value in dropdown not clonable"),
                // id.downcast_ref::<String>()?.clone(),
            })
        }).collect::<Vec<_>>();
        let displays = items.iter().map(|i| i.display.clone()).collect();

        *self = Self::Built {
            items,
            cached_displays: displays
        };
    
        Ok(())
    }

    fn get_displays(&self) -> Cow<'_, Vec<String>> {
        match self {
            Self::Static(items) => Cow::Borrowed(items),
            Self::Variable(_) => Cow::Owned(Vec::new()),
            Self::Built { cached_displays, ..} => Cow::Borrowed(cached_displays),
        }
    } 

    fn len(&self) -> usize {
        match self {
            Self::Static(items) => items.len(),
            Self::Variable(_) => 0,
            Self::Built { items, .. } => items.len()
        }
    }
}
impl From<Vec<String>> for DropdownVariants {
    fn from(value: Vec<String>) -> Self {
        Self::Static(value)
    }
}
impl From<String> for DropdownVariants {
    fn from(value: String) -> Self {
        Self::Variable(value)
    }
}
impl From<DropdownBuilderVariants> for DropdownVariants {
    fn from(value: DropdownBuilderVariants) -> Self {
        match value {
            DropdownBuilderVariants::Static(items) => Self::Static(items),
            DropdownBuilderVariants::Variable(var) => Self::Variable(var),
        }
    }
}

pub struct DropdownWrapper {
    id: String,
    value: Box<dyn Reflect>,
    display: String,
}
impl core::fmt::Display for DropdownWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.display.fmt(f)
    }
}
impl Clone for DropdownWrapper {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            // unwrap is fine here because it needed to be cloned to get here in the first place
            value: self.value.duplicate().unwrap(), 
            display: self.display.clone()
        }
    }
}


pub enum DropdownValue {
    Index(Option<usize>),
    Variable(String, Option<usize>),
}
impl DropdownValue {
    fn index(&self) -> Option<usize> {
        match self {
            Self::Index(n) => *n,
            Self::Variable(_, n) => *n,
        }
    }

    fn set_index(&mut self, index: usize) {
        match self {
            Self::Index(n) => *n = Some(index),
            Self::Variable(_, n) => *n = Some(index),
        }
    }
}
impl From<Option<usize>> for DropdownValue {
    fn from(value: Option<usize>) -> Self {
        Self::Index(value)
    }
}
impl From<String> for DropdownValue {
    fn from(value: String) -> Self {
        Self::Variable(value, None)
    }
}
impl From<DropdownBuilderValue> for DropdownValue {
    fn from(value: DropdownBuilderValue) -> Self {
        match value {
            DropdownBuilderValue::Index(i) => Self::Index(i),
            DropdownBuilderValue::Variable(var) => var.into(),
        }
    }
}
