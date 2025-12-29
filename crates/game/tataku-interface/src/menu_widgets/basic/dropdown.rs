use crate::prelude::*;
use common::reflect::*;
use tataku::TatakuValue;
use ui::{
    tree::*,
    widget::*,
    style::*,
    message::*,
};
use input::{
    InputType,
    InputEvent,
    MouseButton,
};

#[derive(ChainableInitializer)]
pub struct Dropdown {
    #[chain] placeholder: widgets::WidgetText,
    value: DropdownValue,

    container: NodeId,
    variants: DropdownVariants,
    main_button: DropdownButton,

    on_change: DropdownOnChange,

    width: f32,
    node_id: NodeId,
}
impl Dropdown {
    pub fn new(
        variants: DropdownVariants,
        value: DropdownValue,
        on_change: DropdownOnChange,
        placeholder: widgets::WidgetText,
    ) -> Self {
        let main_button = widgets::Button::new(widgets::Text::new(placeholder.clone()))
            .into_widget_base();

        Self {
            value,

            placeholder,

            container: ui::EMPTY_NODE,
            variants,
            main_button,

            on_change,

            // theme: DropdownTheme::sane_defaults(),

            width: 0.0,
            node_id: ui::EMPTY_NODE
        }
    }

    fn set_value(
        &mut self,
        index: usize,
        shell: &mut MessageShell<actions::Action>
    ) {
        let DropdownVariants::Buttons { enum_values, enum_variants, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        shell.state_mut(self.node_id).unwrap().set_active(false);
        self.value.set_index(index);

        self.main_button.inner.child.text.set(enum_variants[index].clone());

        // debug!("setting value to {index} ({})", self.variants.get_displays()[index]);

        let message = match &self.on_change {
            DropdownOnChange::Buildable(actions) => {
                let passed_in = enum_values[index].duplicate().unwrap();
                let passed_in = Some(&TatakuValue::Reflect(passed_in));

                // info!("set value: {passed_in:?}");

                // todo: error on bad
                let actions = actions.iter()
                    .cloned()
                    .filter_map(|action| action.resolve(
                        self.node_id, 
                        shell.source, 
                        shell.values, 
                        passed_in
                    ));

                shell.actions.extend(actions);

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
impl Widget<actions::Action> for Dropdown {
    fn name(&self) -> CowStr { "dropdown_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        let DropdownVariants::Buttons { buttons, .. } = &self.variants else {
            unreachable!("dropdown variants are built");
        };

        let buttons = buttons.iter()
            .map(|button| button as &dyn Widget<_>)
            .collect();

        WidgetChildren::OwnedList(buttons)
    }

    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        let DropdownVariants::Buttons { buttons, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        let buttons = buttons.iter_mut()
            .map(|button| button as &mut dyn Widget<_>)
            .collect();

        WidgetChildrenMut::OwnedList(buttons)
    }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<actions::Action>
    ) -> taffy::TaffyResult<NodeId> {
        let node_id = shell.tree.new_leaf()?;
        self.node_id = node_id;

        let source = shell.source;
        self.main_button.inner.on_press_left = Some(Box::new(move || Some(Message::new(
            source,
            "toggle_dropdown",
            Some(MessageTarget::Node(node_id)),
            Box::new(()),
        ))).into());

        if let Err(e) = self.variants.build(self.node_id, source, shell.values) {
            error!("error building variants: {e:?}");
            self.variants = DropdownVariants::Buttons {
                buttons: Vec::new(),
                enum_variants: Vec::new(),
                enum_values: Vec::new(),
            };
        }

        let DropdownVariants::Buttons { buttons, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        let children = buttons
            .iter_mut()
            .map(|text| text.layout(shell))
            .collect::<taffy::TaffyResult<Vec<_>>>()?;

        self.container = shell.tree.new_with_children(&children)?;

        shell.with_context(self.container, |ctx| {
            ctx.element_data = ElementData {
                element_name: "column".into(),
                ..Default::default()
            }
        });

        if let Some(index) = self.value.index() {
            self.main_button.inner.child.text = buttons[index].inner.child.text.clone();
        }

        self.main_button.layout(shell)?;

        shell.tree.add_child(self.node_id, self.main_button.node_id());
        shell.tree.add_child(self.node_id, self.container);

        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });

        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        let DropdownVariants::Buttons { buttons, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        for variant in buttons.iter_mut() {
            variant.init_style(shell);
        }

        let styles = shell.resolver.resolve_style(
            "",
            self.container,
            shell.tree,
        );

        let ctx = shell.tree.get_context_mut(self.container).unwrap();
        ctx.set_styles(styles, shell.values);

        self.main_button.init_style(shell);

        // Wait until the tree is recomputed to set min and max widths
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        let DropdownVariants::Buttons { buttons, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        let Some(state) = shell.state(self.node_id) 
        else { return };
        let active = state.active();

        self.main_button.input(event, shell);

        if active {
            for variant in buttons {
                if shell.event_consumed { return; }

                variant.input(event, shell);
            }
        }
        if shell.event_consumed { return; }

        let state = shell.state_mut(self.node_id).unwrap();
        match &event.event {
            InputType::KeyPress(input) if active => {
                let Some(key) = input.key else { return };

                if key == input::Key::Escape {
                    state.set_active(false);

                    shell.event_consumed = true;
                }
            }

            InputType::MousePress(MouseButton::Left) if active => {
                state.set_active(false);
            }
            InputType::MousePressCancel(MouseButton::Left) if active => {
                state.set_active(false);
            }

            _ => {}
        }
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        if self.value.index().is_none() {
            self.main_button.inner.child.text = self.placeholder.clone();
        }

        self.main_button.update(shell);

        let DropdownVariants::Buttons { buttons, enum_variants, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        for variant in buttons {
            variant.update(shell);
        }

        let Some(button) = shell.tree.get_layout(self.main_button.node_id()) else { return; };
        let height = button.size.height;

        let Some(container) = shell.tree.get_layout(self.container) else { return; };
        let width = container.size.width;

        if (self.width - width).abs() > f32::EPSILON {
            self.width = width;

            shell.tree.update_style(self.node_id, |style| {
                style.width = CssValue::Value(CssUnit::Pixels(f16::from_f32(width)));
            });

            // todo: move to a more appropriate place
            shell.tree.update_style(self.container, |style| {
                style.margin_top = CssValue::Value(CssUnit::Pixels(f16::from_f32(height)));
            });
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
                .map(TatakuValue::from_reflection)
            {
                Ok(Ok(TatakuValue::U32(index))) => Some(index as usize),
                Ok(Ok(TatakuValue::U64(index))) => Some(index as usize),
                Ok(Ok(TatakuValue::String(enum_variant))) => {
                    enum_variants.iter().position(|x| x == &enum_variant)
                },
                Ok(Ok(TatakuValue::Reflect(reflect))) => match reflect.reflect_display(ReflectPath::EMPTY, None) {
                    Ok(enum_variant) => {
                        enum_variants.iter().position(|x| x == &enum_variant)
                    },
                    Err(e) => {
                        error!("dropdown error: {e:?}");
                        return
                    },
                }
                Ok(Ok(value)) => {
                    error!("bad dropdown value: {value:?}");
                    return
                }
                Ok(Err(ReflectError::OptionIsNone)) => None,
                Ok(Err(e)) => {
                    error!("dropdown error: {e:?}");
                    return
                }
                Err(ReflectError::EntryNotExist { .. }) => None,
                Err(e) => {
                    error!("dropdown error: {e:?}");
                    return
                }
            };

            if *index != selected && let Some(selected) = selected {
                self.main_button.inner.child.text.set(enum_variants[selected].clone());
            }

            *index = selected;
        }
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
    ) {
        let DropdownVariants::Buttons { buttons, .. } = &mut self.variants else {
            unreachable!("dropdown variants are built");
        };

        if shell.handled { return };

        self.main_button.handle_message(message, shell);

        for variant in buttons {
            variant.handle_message(message, shell);
        }

        if !matches!(message.target, Some(MessageTarget::Node(n)) if n == self.node_id) { return; }

        match &*message.tag {
            "select_index" => {
                let Some(index) = message.value.downcast_ref::<usize>().copied() else { return; };

                self.set_value(index, shell);

                shell.handled = true;
            },

            "toggle_dropdown" => {
                let Some(state) = shell.state_mut(self.node_id) 
                else { return };

                state.toggle(ElementState::Active);

                shell.handled = true;
            },

            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        self.main_button.draw(shell);
    }

    fn draw_overlay(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(state) = shell.state(self.node_id) 
        else { return };
        if !state.active() { return }

        let DropdownVariants::Buttons { 
            buttons, .. 
        } = &self.variants else {
            unreachable!("dropdown variants are built");
        };

        for variant in buttons {
            variant.draw(shell);
        }
    }
}


type OnChange = Box<dyn Fn(usize) -> Message + Send + Sync>;
pub enum DropdownOnChange {
    Buildable(Vec<BuildableAction>),
    Callback(OnChange),
}
impl From<OnChange> for DropdownOnChange {
    fn from(value: OnChange) -> Self {
        Self::Callback(value)
    }
}
impl From<BuildableAction> for DropdownOnChange {
    fn from(action: BuildableAction) -> Self {
        vec![action].into()
    }
}
impl From<Vec<BuildableAction>> for DropdownOnChange {
    fn from(mut actions: Vec<BuildableAction>) -> Self {
        for action in actions.iter_mut() {
            if let BuildableAction::Conditional {
                cond,
                ..
            } = action {
                cond.build();
            }
        }

        Self::Buildable(actions)
    }
}

type DropdownButton = widgets::WidgetBase<widgets::Button<widgets::Text>>;

pub enum DropdownVariants {
    Variable(engine::VariablePathResolver),
    Buttons {
        buttons: Vec<DropdownButton>,
        enum_variants: Vec<String>,
        enum_values: Vec<Box<dyn Reflect>>,
    },
}
impl DropdownVariants {
    fn build(&mut self, dropdown: NodeId, source: MessageSource, values: &dyn Reflect) -> tataku::Result<()> {
        let Self::Variable(var) = self
        else { return Ok(()) };

        // FIXME: for some reason when building settings these can be the wrong type ????
        // println!("building variants");

        // println!("resolving path: {var}");

        // if var.as_ref().starts_with("::_setting.type.Dropdown") {
        //     if let Ok(d) = values.reflect_get::<engine::settings::BuildableSetting>("_setting") {
        //         println!("actual type: {d:?}");
        //     }
        // }

        let path = var.resolve_path(values)?;
        // println!("got path: {path}");

        let mut buttons = Vec::new();
        let mut enum_variants = Vec::new();
        let mut enum_values = Vec::new();

        for (index, variant) in values.reflect_iter(&*path)?.enumerate() {
            let value = match variant.index {
                // This may be useful even though it currently is not implemented this way
                // Some(ReflectItemIndex::Number(index)) => {
                //     Box::new(index)
                // },
                Some(ReflectItemIndex::Value(reflect)) => {
                    reflect.duplicate().unwrap()
                },
                _ => {
                    variant.duplicate().unwrap()
                },
            };
            let display = variant.reflect_display(ReflectPath::EMPTY, None)?;

            enum_values.push(value);

            match variant.index {
                Some(ReflectItemIndex::Number(_n)) => {
                    enum_variants.push(display.clone());
                },
                Some(ReflectItemIndex::Value(v)) => {
                    let s = v.downcast_ref::<String>()
                        .ok_or_else(|| format!("{path} not string indexable!"))?;

                    enum_variants.push(s.clone());
                },
                None => {
                    return Err(format!("{path} not indexable!").into());
                },
            }

            let callback = Box::new(move || {
                Some(Message::new(
                    source,
                    "select_index",
                    Some(MessageTarget::Node(dropdown)),
                    Box::new(index),
                ))
            });

            let button = widgets::Button::new(widgets::Text::new(display.into()))
                .on_press_left(Some(widgets::ButtonOnClick::Callback(callback)))
                .into_widget_base();

            buttons.push(button);
        }

        *self = Self::Buttons {
            buttons,
            enum_variants,
            enum_values,
        };

        Ok(())
    }
}
impl From<engine::VariablePathResolver> for DropdownVariants {
    fn from(value: engine::VariablePathResolver) -> Self {
        Self::Variable(value)
    }
}

#[derive(Debug)]
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
    Variable(engine::VariablePathResolver, Option<usize>),
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
impl From<engine::VariablePathResolver> for DropdownValue {
    fn from(value: engine::VariablePathResolver) -> Self {
        Self::Variable(value, None)
    }
}
