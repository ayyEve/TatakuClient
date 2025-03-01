use crate::prelude::*;
use crate::prelude::ui::*;

const FILTERED_TEXT_PATH: &str = "var.settings.search_filter";

pub struct SettingsMenu {
    filter_text: String,
    old_settings: Settings,

    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl SettingsMenu {
    pub fn new(settings: &Settings) -> Self {
        Self {
            filter_text: String::new(),
            old_settings: settings.clone(),

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        }
    }

    fn view(
        &self, 
        values: &mut dyn Reflect, 
        owner: MessageOwner
    ) -> Box<dyn Widget> {
        let game_time = values.reflect_get::<f32>("game.time")
            .unwrap_or(MaybeOwned::Owned(0.0))
            .copied();


        // set/clear the entry for the settings filter in the map
        values.reflect_insert(
            FILTERED_TEXT_PATH,
            ItemFilter::new(
                Vec::new(), 
                QueryType::Any
            )
        ).expect("couldnt insert settings filter into dynmap");

        let settings = values.reflect_get::<Settings>("settings").unwrap().cloned();

        // build settings list
        let mut builder = SettingsBuilder {
            values,
            categories: Vec::new(),

            create_empty: Box::new(|| EmptyWidget::new_boxed()),
            create_text: Box::new(|builder| {
                TextWidget::new(builder.text)
                    .chain_maybe(builder.font_size, |s, f| s.font_size(f))
                    .boxed()
            }),
            create_checkbox: Box::new(|builder| {
                Checkbox::new(builder.text, builder.value)
                    .chain_maybe(builder.font_size, |s, f| s.font_size(f))
                    .chain_maybe(builder.on_change, |s, f| s.on_toggle_arced(f))
                    .boxed()
            }),
            create_slider: Box::new(|builder| {
                Slider::new(
                    builder.range, 
                    builder.value,
                    builder.on_change
                )
                    .step_maybe(builder.step)
                    .boxed()
            }),
            create_text_input: Box::new(|builder| {
                TextInput::new(builder.placeholder, builder.value)
                    .chain_maybe(builder.font_size, |s, f| s.font_size(f))
                    .on_input(builder.on_input)
                    .on_submit(builder.on_submit)
                    .secure(builder.secure)
                    .boxed()
            }),
            create_button: Box::new(|builder| {
                Button::new(builder.element)
                    .on_press(builder.on_press)
                    .boxed()
            }),
            create_dropdown: Box::new(|builder| {
                Dropdown::new(builder.variants, builder.value, builder.on_change)
                    .chain_maybe(builder.font_size, |s, f| s.font_size(f))
                    .boxed()
            }),
        };
        settings.into_elements(
            "settings".to_owned(), 
            owner, 
            &mut builder
        );

        // TODO: hide catergory names when all items are filtered out
        let items = builder.categories
            .into_iter()
            .filter(|sc| !sc.properties.is_empty())
            .flat_map(|sc| [
                // space
                row!( Space::new(FILL, Dimension::Length(40.0)).boxed(); ),
                // category name
                row!( TextWidget::new(sc.name).font_size(40.0).boxed(); ),
                // settings
                Container::new(sc.properties
                    .into_iter()
                    .zip(sc.values)
                    .map(|(p, v)| 
                        Container::new(vec![p, v])
                        .vertical_align(AlignContent::Center)
                        .horizontal_align(AlignContent::SpaceBetween)
                        .margin([LengthPercentageAuto::Length(0.0), LengthPercentageAuto::Length(5.0)])
                        .width(FILL)
                        .boxed()
                    )
                    .zip(sc.names)
                    .map(|(w, name)| 
                        FilterableWidget::new(
                            w,
                            name,
                            FILTERED_TEXT_PATH.to_owned()
                        ).boxed()
                    )
                    .collect()
                )
                .flex_direction(FlexDirection::Column)
                // .spacing(LengthPercentage::Length(5.0))
                .margin([ LengthPercentageAuto::Length(0.0), LengthPercentageAuto::Length(5.0) ])
                .width(FILL)
                .boxed()
            ]
        ).collect();
        
        let everything = Container::new(vec![
            TextWidget::new("Settings").font_size(40.0).boxed(),

            // space
            Space::new(FILL, Dimension::Length(10.0)).boxed(),

            // search text
            TextInput::new("Search", self.filter_text.clone())
                .font_size(30.0)
                .on_input(move |t: &str| Message::new(owner, "search", MessageValue::Text(t.to_string())))
                .boxed(),

            // space
            Space::new(FILL, Dimension::Length(40.0)).boxed(),

            // items
            Container::new(items)
                .id("settings_list")
                .scrollable(true)
                .drag_scroll(true)
                .flex_direction(FlexDirection::Column)
                .vertical_overflow(taffy::Overflow::Scroll)
                .width(FILL)
                .boxed()
            ,

            // revert
            Button::new(TextWidget::new("Revert").boxed())
                .on_press(Message::new(owner, "revert", MessageValue::Click))
                .boxed(),

            // done
            Button::new(TextWidget::new("Done").boxed())
                .on_press(Message::new(owner, "done", MessageValue::Click))
                .boxed()
        ])
        .width(FILL)
        .height(FILL)
        .flex_direction(FlexDirection::Column)
        .boxed();

        TransformableWidget::new(
            vec![
                AnimatableTrigger { 
                    trigger: AnimatableTriggerEvent::Message(MessageTag::String("close".to_owned())), 
                    action: "close_dialog".to_owned()
                }
            ],
            [("close_dialog".to_owned(), vec![AnimatableAction {
                action: TransformType::VectorScale { start: Vector2::new(1.0, 1.0), end: Vector2::new(0.0, 1.0) },
                start: AnimatableTransformValue::Current,
                stop: AnimatableTransformValue::Current,
                duration: 200.0,
            }])].into_iter().collect(),
            everything
        )
        .with_transform(Transformation::new(
            0.0,
            200.0,
            TransformType::VectorScale { start: Vector2::new(0.0, 1.0), end: Vector2::new(1.0, 1.0) },
            Easing::Linear,
            game_time 
        ))
        .width(FILL)
        .height(FILL)
        .vertical_overflow(taffy::Overflow::Scroll)
        .boxed()
    }
}

#[async_trait]
impl Widget for SettingsMenu {
    fn name(&self) -> Cow<'static, str> { "settings_menu".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.view(shell.values, shell.owner);
        let child = self.node.layout(shell)?;

        self.node_id = shell.tree.new_with_children(
            Style {
                size: Size::auto(),
                ..menu_layout()
            },
            &[ child ]
        )?;
        
        Ok(self.node_id)
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>,
        actions: &mut ActionQueue
    ) {
        self.node.update(shell, actions);
    }
    
    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<'_>,
    ) {
        self.node.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.node.draw(shell);
    }

    
    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        let Some(tag) = message.tag.as_string() else { return };
        
        let mut tags = ReflectPath::new(tag);
        let Some(first) = tags.next() else { return warn!("no first?") };

        let settings = values
            .reflect_get_mut::<Settings>("settings")
            .unwrap();

        match first {
            "done" => {
                settings.check_hashes();
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            },
            "revert" => {
                *settings = self.old_settings.clone();
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            },
            "search" => if let Some(text) = message.value.as_text_ref() { 
                let filter = ItemFilter::new(
                    text.clone().split(" ").map(String::from).collect(), 
                    QueryType::Any
                );

                if let Err(e) = values.reflect_insert(FILTERED_TEXT_PATH, Box::new(filter)) {
                    panic!("{e:?}")
                }
            },

            // graceful close requested
            "close" => {
                // run the close animation
                self.node.handle_message(
                    &Message::new(message.owner, "close", MessageValue::Click), 
                    values, 
                    actions
                ).await;
                let close_task = ActionTask::new(UiAction::new(self.node_id, DialogAction::Close));
                let task = DelayTask::new(close_task, 200);
                actions.push(TaskAction::AddTask(Box::new(task)));
            }

            "var" => {
                let mut settings = (*settings).clone();
                settings.gamemode_settings.from_elements(
                    &mut tags, 
                    message.clone(), 
                    &mut FromElementsExtra { 
                        values
                    }
                );
                values.reflect_insert("settings", settings).unwrap();
            }

            _ => {
                let mut settings = (*settings).clone();
                settings.from_elements(
                    &mut tags, 
                    message.clone(), 
                    &mut FromElementsExtra { 
                        values
                    }
                );
                values.reflect_insert("settings", settings).unwrap();
            }
        }

    }
    
}
