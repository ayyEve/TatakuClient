use crate::prelude::*;

// TODO: add spacing between dropdown items

#[derive(ChainableInitializer)]
pub struct Dropdown {
    #[chain] placeholder: DropdownPlaceholder,
    value: DropdownValue,
    variants: DropdownVariants,

    on_change: DropdownOnChange,
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
        placeholder: impl Into<DropdownPlaceholder>,
    ) -> Self {
        let variants = variants.into();
        let value = value.into();

        Self {
            value,

            placeholder: placeholder.into(),
            variants,
            on_change: on_change.into(),
            active: false,
            hover: false,
            active_index: None,

            // theme: DropdownTheme::sane_defaults(),

            node_id: EMPTY_NODE
        }
    }

    fn get_style(&self, text_style: &TextStyle, scale: Option<Vector2>) -> (CssUnit, CssUnit) {
        let placeholder_size = text_style
            .measure_text(self.placeholder.get(), scale);
        
        let largest_text = self.variants.get_displays()
            .iter()
            .map(|a| text_style.measure_text(a, scale))
            .fold(
                placeholder_size, 
                |a, b| Vector2::new(a.x.max(b.x), a.y.max(b.y))
            );
        
        let min_width = CssUnit::Pixels(half::f16::from_f32(largest_text.x));
        let min_height = CssUnit::Pixels(half::f16::from_f32(largest_text.y));
        (min_width, min_height)
    }

    fn set_value(
        &mut self, 
        index: usize, 
        shell: &mut InputShell<TatakuAction>
    ) {
        self.active = false;
        self.value.set_index(index);
        // debug!("setting value to {index} ({})", self.variants.get_displays()[index]);

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
                    } => TatakuValue::from_reflection(
                        items[index].value.duplicate().unwrap()
                    ).inspect_err(|e| 
                        warn!("didnt reflect: {e:?}")
                    )
                    .ok(),
                };

                let action = buildable_action
                    .clone()
                    .into_action(
                    self.node_id, 
                    shell.values, 
                    passed_in.as_ref(),
                );
                if let Some(action) = action {
                    shell.actions.push(action);
                }

                None
            }

            DropdownOnChange::Callback(f) 
                => Some(f(index)),
        };

        if let Some(m) = message {
            shell.messages.push(m);
        }
    }
}
impl Widget<TatakuAction> for Dropdown {
    fn name(&self) -> CowStr { "dropdown_widget".into() }
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

        let (w, h) = self.get_style(text_style, None);
        shell.tree.update_style(
            self.node_id, 
            |style| {
                style.min_width = w.into();
                style.min_height = h.into();
            }
        );
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<TatakuAction>,
    ) {
        let Some(bounds) = shell.tree.bounds(self.node_id) 
        else { return };

        let Some(context) = shell.tree.get_context(self.node_id) 
        else { return };

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
                self.active = self.hover;
                if self.active {
                    shell.event_consumed = true;
                }
            }

            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        let theme = &shell.general_theme;
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        // bounding box
        shell.list.push(
            Rectangle::new_bounds(
                bounds,
                theme.background_color,
            ).border(Border::new(
                theme.get_color(self.active, self.hover), 
                2.0
            ))
        );

        // selected text
        let displays = self.variants.get_displays();
        let main_text = self.value.index()
            .and_then(|n| displays.get(n).map(|s| s.as_str()))
            .unwrap_or(self.placeholder.get());

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        shell.list.push(text_style.create_text(main_text.to_string(), bounds));
    }

    fn draw_overlay(&self, shell: &mut DrawShell<TatakuAction>) {
        if !self.active { return }
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };
        let theme = &shell.general_theme;

        let selected = self
            .value
            .index()
            .unwrap_or(self.variants.len());

        let active = self
            .active_index
            .unwrap_or(self.variants.len());

        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        // draw all options
        // TODO: margin between items
        for (n, i) in self
            .variants
            .get_displays()
            .iter()
            .cloned()
            .enumerate() 
        {
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
                ).border(Border::new(
                    theme.get_color(n == selected, n == active), 
                    2.0
                ))
            );

            let text = text_style.create_text(
                i, 
                Bounds::new(offset, bounds.size)
            );
            shell.list.push(text);
        }
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        self.placeholder.update(shell.values);
        if self.variants.is_unbuilt() {
            if let Err(e) = self.variants.build(shell.values) {
                error!("error building variants: {e:?}");
                self.variants = DropdownVariants::Static(vec!["ERROR".to_string()]);
                
                return;
            }
            let text_style = shell.tree
                .get_text_style(self.node_id)
                .unwrap();

            let (w, h) = self.get_style(text_style, None);
            shell.tree.update_style(
                self.node_id, 
                |style| {
                    style.min_width = w.into();
                    style.min_height = h.into();
                }
            );
        }

        if let DropdownValue::Variable(
            var, 
            index
        ) = &mut self.value {

            let path = match var.resolve_path(shell.values) {
                Ok(p) => p,
                Err(e) => {
                    error!("error with path variable '{var:?}': {e:?}");
                    return
                }
            };


            let selected = match shell
                .values
                .impl_get(ReflectPath::new(&path))
            {
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


                let DropdownVariants::Built { 
                    items, 
                    .. 
                } = &self.variants else { return };

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

pub enum DropdownVariants {
    Static(Vec<String>),
    Variable(VariablePathResolver),
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
        let Self::Variable(var) = self 
        else { return Ok(()) };

        let var = var.resolve_path(values)?;

        let iter = values.reflect_iter(&*var)?;
        let items = iter.filter_map(|value| {
            let id = TatakuValue::from_reflection(value.item).ok()?.as_string();
            Some(DropdownWrapper {
                display: value
                    .impl_display(ReflectPath::new(""), None)
                    .unwrap_or_else(|_| id.clone()), 
                id,
                value: value
                    .duplicate()
                    .expect("Value in dropdown not clonable"),
            })
        }).collect::<Vec<_>>();

        let displays = items
            .iter()
            .map(|i| i.display.clone())
            .collect();

        *self = Self::Built {
            items,
            cached_displays: displays
        };
    
        Ok(())
    }

    fn get_displays(&self) -> Cow<'_, [String]> {
        match self {
            Self::Static(items) => Cow::Borrowed(items),
            Self::Variable(_) => Cow::Owned(Vec::new()),
            Self::Built { 
                cached_displays, 
                ..
            } => Cow::Borrowed(cached_displays),
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
impl From<ArcStr> for DropdownVariants {
    fn from(value: ArcStr) -> Self {
        Self::Variable(VariablePathResolver::new(value))
    }
}
impl From<String> for DropdownVariants {
    fn from(value: String) -> Self {
        Self::Variable(value.into())
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
    Variable(VariablePathResolver, Option<usize>),
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
        Self::Variable(VariablePathResolver::new(value), None)
    }
}
impl From<ArcStr> for DropdownValue {
    fn from(value: ArcStr) -> Self {
        Self::Variable(VariablePathResolver::new(value), None)
    }
}

#[derive(Default2)]
pub enum DropdownPlaceholder {
    #[default]
    Static(ArcStr),
    Buildable {
        buildable: BuildableText,
        cache: String
    }
}
impl DropdownPlaceholder {
    fn get(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Buildable { cache, .. } => cache,
        }
    }
    fn update(&mut self, values: &dyn Reflect) {
        let Self::Buildable { buildable, cache } = self 
        else { return };

        *cache = buildable.to_string(values);
    }
}
impl From<ArcStr> for DropdownPlaceholder {
    fn from(value: ArcStr) -> Self {
        Self::Static(value)
    }
}
impl From<String> for DropdownPlaceholder {
    fn from(value: String) -> Self {
        Self::Static(value.into())
    }
}
impl From<BuildableText> for DropdownPlaceholder {
    fn from(mut value: BuildableText) -> Self {
        if let Err(e) = value.compute() {
            error!("Error building text: {e:?}");
        }
        Self::Buildable { 
            buildable: value, 
            cache: String::new() 
        }
    }
}
