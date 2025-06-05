use crate::prelude::*;
use crate::prelude::ui::*;

const FILTERED_TEXT_PATH: &str = "var.settings.search_filter";

pub struct SettingsMenu {
    filter_text: String,
    old_settings: Settings,

    node: Box<dyn Widget>,
}
impl SettingsMenu {
    pub const DEFAULT_OPTIONS: DialogCreateOptions = DialogCreateOptions {
        allow_multiple: false,
        resizable: false,
        draggable: false,
        title: Cow::Borrowed("Settings"),
        location: DialogLocation::Auto,
        background: true,
    };

    pub fn new(settings: &Settings) -> Self {
        Self {
            filter_text: String::new(),
            old_settings: settings.clone(),

            node: EmptyWidget::new_boxed(),
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

        let settings = values.reflect_get::<Settings>("settings")
            .unwrap()
            .cloned();

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
            create_key_button: Box::new(|builder| {
                KeyButton::new(builder.value)
                    .on_change(builder.on_change)
                    .optional(builder.optional)
                    .boxed()
            }),


        };
        settings.into_elements(
            "settings".to_owned(), 
            owner, 
            &mut builder
        );

        // TODO: hide category names when all items are filtered out
        let items = builder.categories
            .into_iter()
            .filter(|sc| !sc.properties.is_empty())
            .flat_map(|sc| [
                // space
                Container::new(Vec::new())
                    .flex_direction(FlexDirection::Row)
                    .width(FILL)
                    .boxed(),

                // category name
                row!( TextWidget::new(sc.name).font_size(40.0).boxed(); ),
                // settings
                Container::new(sc
                    .properties
                    .into_iter()
                    .zip(sc.values)
                    .map(|(p, v)| 
                        Container::new(vec![p, v])
                        .vertical_align(AlignContent::Center)
                        .horizontal_align(AlignContent::SpaceBetween)
                        .margin([
                            LengthPercentageAuto::Length(5.0), 
                            LengthPercentageAuto::Length(5.0)
                        ])
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
                .margin(LengthPercentageAuto::Length(5.0))
                .width(FILL)
                .boxed()
            ]
        ).collect();
        
        let everything = Container::new(vec![
            TextWidget::new("Settings").font_size(40.0).boxed(),

            // // space
            // Space::new(FILL, Dimension::Length(10.0)).boxed(),

            // search text
            TextInput::new("Search", self.filter_text.clone())
                .font_size(30.0)
                .on_input(move |t: &String| Message::new(
                    owner, 
                    "search", 
                    MessageValue::Text(t.to_string())
                ))
                .boxed(),

            // // space
            // Space::new(FILL, Dimension::Length(40.0)).boxed(),

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
                .on_press(Message::new(owner, "close", MessageValue::Click))
                .boxed()
        ])
        .width(FILL)
        .height(FILL)
        .flex_direction(FlexDirection::Column)
        .boxed();

        TransformableWidget::new(
            vec![
                AnimatableTrigger { 
                    trigger: AnimatableTriggerEvent::Message(
                        MessageTag::String("close".to_owned())
                    ), 
                    action: "close".to_owned()
                }
            ],
            [("close".to_owned(), vec![AnimatableAction {
                action: TransformTypeTag::VectorScale { 
                    start: Vector2::new(1.0, 1.0), 
                    end: Vector2::new(0.0, 1.0) 
                },
                duration: 200.0,
            }])].into_iter().collect(),
            everything
        )
        .with_transform(Transformation::new(
            0.0,
            200.0,
            TransformType::VectorScale { 
                start: Vector2::new(0.0, 1.0), 
                end: Vector2::new(1.0, 1.0) 
            },
            Easing::Linear,
            game_time 
        ))
        .min_width(Dimension::Percent(0.25))
        .max_width(Dimension::Percent(0.75))
        .height(FILL)
        .vertical_overflow(taffy::Overflow::Scroll)
        .boxed()
    }
}
impl Widget for SettingsMenu {
    fn name(&self) -> Cow<'static, str> { "settings_menu".into() }
    fn node_id(&self) -> NodeId { self.node.node_id() }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell, 
        display_override: Option<ui::Display>
    ) {
        self.node.update_styles(shell, display_override);
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        self.node = self.view(shell.values, shell.owner);
        self.node.layout(shell)
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        self.node.update(shell);
    }
    
    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        self.node.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell) {
        self.node.draw(shell);
    }

    fn draw_overlay(&self, shell: &mut DrawShell) {
        self.node.draw_overlay(shell);
    }

    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        let Some(tag) = message.tag.as_string() else { return };
        
        let mut tags = ReflectPath::new(tag);
        let Some(first) = tags.next() else { return  };

        let settings = shell.values
            .reflect_get_mut::<Settings>("settings")
            .unwrap();

        match first {
            "revert" => {
                shell.handled = true;
                *settings = self.old_settings.clone();
            },
            "search" => if let Some(text) = message.value.as_text_ref() { 
                shell.handled = true;
                let filter = ItemFilter::new(
                    text.clone().split(" ").map(String::from).collect(), 
                    QueryType::Any
                );

                if let Err(e) = shell.values.reflect_insert(
                    FILTERED_TEXT_PATH, 
                    Box::new(filter)
                ) {
                    panic!("{e:?}")
                }
            },

            // graceful close requested
            "force_close" | "close" => {
                shell.handled = true;
                settings.check_hashes();

                // run the close animation
                self.node.handle_message(
                    &Message::new(
                        message.owner, 
                        "close", 
                        MessageValue::Click
                    ), 
                    shell
                );
                let close_task = ActionTask::new(
                    ActionTaskAction::Action(UiAction::new(
                        self.node_id(), 
                        DialogAction::Close,
                    ).into())
                );

                let task = DelayTask::new(close_task, 200);
                shell.actions.push(TaskAction::AddTask(Box::new(task)));
            }

            "var" => {
                shell.handled = true;
                let mut settings = (*settings).clone();
                settings.gamemode_settings.from_elements(
                    &mut tags, 
                    message.clone(), 
                    &mut GenericShell {
                        tree: shell.tree,
                        values: shell.values,
                        messages: shell.messages,
                        actions: shell.actions,
                    },
                );
                shell.values
                    .reflect_insert("settings", settings)
                    .unwrap();
            }

            _ => {
                shell.handled = true;
                let mut settings = (*settings).clone();
                settings.from_elements(
                    &mut tags, 
                    message.clone(), 
                    &mut GenericShell {
                        tree: shell.tree,
                        values: shell.values,
                        messages: shell.messages,
                        actions: shell.actions,
                    },
                );
                shell.values
                    .reflect_insert("settings", settings)
                    .unwrap();
            }
        }

    }
    
}
