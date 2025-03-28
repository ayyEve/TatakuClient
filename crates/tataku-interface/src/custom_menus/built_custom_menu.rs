use crate::prelude::*;
use crate::prelude::ui::*;

pub struct BuiltCustomMenu {
    pub id: String,
    pub element: Box<dyn Widget>,
    pub actions: ActionQueue,
    pub events: HashMap<TatakuEventType, Vec<LuaAction>>,

    node_id: NodeId,
}
impl BuiltCustomMenu {
    pub fn build(menu: &CustomMenu) -> Self {
        let mut events: HashMap<TatakuEventType, Vec<LuaAction>> = HashMap::new();
        for event in menu.events.clone() {
            let list = events.entry(event.event_type).or_default();
            for mut action in event.get_actions() {
                action.build();
                list.push(action);
            }
        }

        Self {
            id: menu.id.clone(),
            element: Self::build_element(menu.element.clone(), MessageOwner::Menu),
            actions: ActionQueue::new(),
            events,
            node_id: EMPTY_NODE,
        }
    }


    fn build_children(children: Vec<ElementDef>, owner: MessageOwner) -> Vec<Box<dyn Widget>> {
        let mut built = Vec::with_capacity(children.len());
        for child in children {
            built.push(Self::build_element(child, owner))
        }
        built
    }

    pub fn build_element(
        element: ElementDef,
        owner: MessageOwner,
    ) -> Box<dyn Widget> {
        let id = element.id;

        match element.element {
            ElementIdentifier::Space => Space::new(element.width, element.height).boxed(),
            ElementIdentifier::Button { 
                padding, 
                action, 
                element: child,
                active_cond
            } => Button::new(Self::build_element(*child, owner))
                .style(element.style)
                .width(element.width)
                .height(element.height)
                .on_press(action)
                .active_condition_maybe(active_cond)
                .chain_maybe(padding, |b, p| b.padding(p))
                .boxed(),
            
            ElementIdentifier::Text { 
                text, 
                color, 
                font_size, 
                font, 
                align,
            } => TextWidget::new(text)
                .style(element.style)
                .width(element.width)
                .height(element.height)
                .font_size_maybe(font_size)
                .chain_maybe(color, |s, c| s.text_color(c))
                .chain_maybe(align, |s, a| s.text_align(a))
                .chain_maybe(font.as_ref().and_then(map_font), |s, font| s.font(font))
                .boxed(),
            
            ElementIdentifier::TextInput { 
                placeholder, 
                variable,  
                on_input, 
                on_submit, 
                is_password 
            } => TextInput::new(placeholder, CustomElementText::Variable(variable))
                .on_input(on_input)
                .style(element.style)
                .width(element.width)
                .secure(is_password)
                .on_submit(on_submit)
                // .chain_maybe(on_submit.as_ref().and_then(|e| e.resolve(owner, shell.values, None)), |t, m| t.on_submit(m))
                .boxed(),
            

            ElementIdentifier::Row { 
                // padding, 
                // margin, 
                elements
            } => Container::new(Self::build_children(elements, owner))
                .style(element.style)
                .id(id)
                .width(element.width)
                .height(element.height)
                .flex_direction(FlexDirection::Row)
                .vertical_overflow(taffy::Overflow::Clip)
                // .chain_maybe(padding, |s, p| s.padding(p))
                // .chain_maybe(margin, |s, m| s.spacing(LengthPercentage::Length(m)))
                .boxed(),
            
            ElementIdentifier::Column { 
                // padding, 
                // margin, 
                elements
            } => Container::new(Self::build_children(elements, owner))
                .style(element.style)
                .id(id)
                .width(element.width)
                .height(element.height)
                .flex_direction(FlexDirection::Column)
                .vertical_overflow(taffy::Overflow::Clip)
                // .chain_maybe(padding, |s, p| s.padding(p))
                // .chain_maybe(margin, |s, m| s.margin(LengthPercentageAuto::Length(m)))
                .boxed(),

            ElementIdentifier::StyledContent { 
                // padding, 
                color, 
                border, 
                shape, 
                element: child,
                image,
            } => ContentBackground::new(Self::build_element(*child, owner))
                .style(element.style)
                .width(element.width)
                .height(element.height)
                .border(border)
                .color(color)
                .image(image)
                .shape_maybe(shape)
                // .chain_maybe(padding, |s, p| s.padding(p))
                .boxed(),
            
            ElementIdentifier::DraggingScroll {
                // padding,
                // margin,
                elements
            } => Container::new(Self::build_children(elements, owner))
                .style(element.style)
                .id(id)
                // .scrollable(true)
                // .drag_scroll(true)
                .flex_direction(FlexDirection::Row)
                .width(element.width)
                .height(element.height)
                // .chain_maybe(padding, |panel, pad| panel.padding(pad))
                // .chain_maybe(margin, |panel, margin| panel.margin(LengthPercentageAuto::Length(margin)))
                .boxed(),

            ElementIdentifier::GameplayPreview { 
                visualization 
            } => {
                let mut gameplay = GameplayPreview::new(true, true, Arc::new(|_| true), owner)
                    // .width(self.width)
                    // .height(self.height)
                    ;

                if let Some(vis) = visualization {
                    match &*vis {
                        "menu_visualization" => gameplay.visualization = Some(MenuVisualization::new()),
                        _ => warn!("Unknown gameplay visualization: {vis}"),
                    }
                }

                gameplay.boxed()
            }

            ElementIdentifier::Animatable {
                triggers,
                actions,
                element
            } => TransformableWidget::new(
                    triggers,
                    actions,
                    Self::build_element(*element, owner)
                )
                .boxed(),

            ElementIdentifier::Conditional { 
                cond, 
                if_true,
                if_false,
            } => ConditionalWidget::new(
                    Self::build_element(*if_true, owner),
                    if_false.map(|f | Self::build_element(*f, owner)),
                    cond
                )
                .style(element.style)
                .boxed(),

            ElementIdentifier::List {
                list_var,
                scrollable,
                variable,
                element: template
            } => Container::new(Vec::new())
                .style(element.style)
                .width(element.width)
                .height(element.height)
                .id(id)
                .scrollable(scrollable).drag_scroll(scrollable)
                .make_programmatic(ProgrammaticListData::new(*template, list_var, variable))
                .vertical_overflow(taffy::Overflow::Clip)
                // .flex_direction(FlexDirection::Column)
                .boxed(),

            ElementIdentifier::Dropdown {
                options_key,
                options_display_key: _,
                selected_key,

                on_select,

                padding,
                placeholder,
                font_size,
                font
            } => Dropdown::new(
                    options_key,
                    selected_key,
                    on_select,
                )
                .chain_maybe(placeholder, |d, p| d.placeholder(p))
                .style(element.style)
                .width(element.width)
                .font_size_maybe(font_size)
                .chain_maybe(padding, |t, p| t.padding(p))
                .chain_maybe(font.as_ref().and_then(map_font), |s, font| s.font(font))
                .boxed(),
        }

        // debug color
        .chain_maybe(element.debug_color, move |e, color| DebugWidget::new(e, color).boxed())
    }
}

#[async_trait]
impl Widget for BuiltCustomMenu {
    fn name(&self) -> Cow<'static, str> { "custom_menu".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        let child = self.element.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            menu_layout(), 
            &[child]
        )?;

        Ok(self.node_id)
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<'_>
    ) {
        self.element.input(event, shell);
    }

    fn draw(
        &self, 
        shell: &mut DrawShell<'_>, 
    ) {
        self.element.draw(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell<'_>, actions: &mut ActionQueue) {
        actions.extend(self.actions.take());
        self.element.update(shell, actions);
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        actions.extend(self.actions.take());

        self.element.handle_message(message, values, &mut self.actions).await;
        if !self.actions.is_empty() { return actions.extend(self.actions.take())}

        let cast = message.value
            .try_downcast_ref::<(CustomMenuAction, Option<TatakuValue>)>()
            .cloned();
        if let Some((action, passed_in)) = cast {
            if let Some(action) = action.into_action(values, passed_in) {
                self.actions.push(action)
            }
            
            return
        }

        let tag = message.tag.clone();
        match message.value.clone() {
            MessageValue::Value(TatakuValue::Reflect(value)) => {
                let Some(variable) = tag.as_string() else { return };
                // values.update_or_insert(&variable, TatakuVariableWriteSource::Menu, incoming, || TatakuVariable::new_any(TatakuValue::None));

                if let Err(e) = values.reflect_insert(variable, value) {
                    error!("error inserting into values: {e:?}");
                }
            }
            MessageValue::Text(incoming) => {
                let Some(variable) = tag.as_string() else { return };
                // values.update_or_insert(&variable, TatakuVariableWriteSource::Menu, incoming, || TatakuVariable::new_any(TatakuValue::None));
                if let Err(e) = values.reflect_insert(variable, Box::new(incoming)) {
                    error!("error inserting into values: {e:?}");
                }

                // values.set(variable, incoming);
            }
            // MessageValue::CustomMenuAction(AddDialog(dialog)) => {}
            // MessageValue::CustomMenuAction(SetMenu(menu)) => self.queued_actions.push(MenuMenuAction::SetMenuCustom(menu)),

            // MessageValue::CustomMenuAction(action, passed_in) => {
            //     if let Some(action) = action.into_action(values, passed_in) {
            //         self.actions.push(action)
            //     }
            // }
            MessageValue::Multi(messages) => {
                for i in messages {
                    self.handle_message(&i, values, actions).await;
                }
            }

            // MessageValue::Custom(a) => {
            //     if let Some(a) = a.downcast_ref::<CustomMenuAction>() {
            //         a.into_action(values, None);
            //     }
            // }

            other => warn!("unhandled message: {other:?}"),
        }
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect,
    ) {
        let Some(events) = self.events.get(&event) else { return };

        for i in events.iter() {
            let Some(message) = i.resolve(MessageOwner::Menu, values, event_value.clone()) else { continue };

            let cast = message.value
                .try_downcast_ref::<(CustomMenuAction, Option<TatakuValue>)>()
                .cloned();
            if let Some((action, passed_in)) = cast {
                let Some(a) = action.into_action(values, passed_in) else { continue };
                self.actions.push(a);
            } else {
                self.actions.push(GameAction::HandleMessage(message))
            }

            // match message.message_type {
            //     MessageValue::CustomMenuAction(action, passed_in) => {
            //         let Some(a) = action.into_action(values, passed_in) else { continue };
            //         self.actions.push(a);
            //     }

            //     _ => self.actions.push(TatakuAction::Game(GameAction::HandleMessage(message))),
            // }
        }

    }

    async fn reload_skin(&mut self, skin: &mut dyn SkinProvider) {
        self.element.reload_skin(skin).await;
    }
}

fn map_font(font: &String) -> Option<Font> {
    match &**font {
        "main" => Some(Font::Main),
        "fallback" => Some(Font::Fallback),
        "fa"|"font_awesome" => Some(Font::FontAwesome),
        _ => None
    }
}
