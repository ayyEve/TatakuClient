use crate::prelude::*;
use crate::prelude::ui::*;

const INPUT_PATH: &str = "var.console_line";
const OUTPUT_PATH: &str = "var.console_output";

pub struct ConsoleDialog {
    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl ConsoleDialog {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        }
    }
}

#[async_trait]
impl Widget for ConsoleDialog {
    fn name(&self) -> Cow<'static, str> { "console_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }


    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.node.update_styles(tree, resolver, display_override);
    }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> ui::TaffyResult<NodeId> {

        if shell.values.reflect_get::<Vec<String>>(OUTPUT_PATH).is_err() {
            shell.values.reflect_insert(OUTPUT_PATH, Vec::<String>::new()).unwrap()
        }
        if shell.values.reflect_get::<String>(INPUT_PATH).is_err() {
            shell.values.reflect_insert(INPUT_PATH, String::new()).unwrap()
        }



        let output = Container::new(Vec::new())
            .make_programmatic(ProgrammaticListData::new(
                Element::Text(Box::new(TextElement {
                    text: BuildableTextInner::Variable("_line".to_string()).into(),
                    ..Default::default()
                })),
                // ElementDef {
                //     id: String::new(),
                //     element: ElementIdentifier::Text {
                //         text: 
                //         color: None,
                //         font_size: None,
                //         font: None,
                //         align: None,
                //     },
                //     debug_color: None,
                //     debug_name: None,
                //     width: Dimension::Percent(1.0),
                //     height: Dimension::Auto,
                //     style: Style::default(),
                // },
                OUTPUT_PATH.to_string(),
                "_line".to_string()
            ))
            .scrollable(true)
            .drag_scroll(true)
            .flex_direction(FlexDirection::Column)
            .width(FILL)
            .height(SHRINK)
            .boxed();

        let input = TextInput::new("Command:", WidgetText::Custom { custom: BuildableTextInner::Variable(INPUT_PATH.to_owned()).as_buildable(), cached: String::new() })
            .on_submit(parse_line(shell.owner))
            .width(FILL)
            .height(SHRINK)
            .boxed();


        self.node = Container::new(vec![output, input])
            .width(FILL)
            .height(FILL)
            .flex_direction(FlexDirection::Column)
            .vertical_align(AlignContent::SpaceBetween)
            .boxed();

        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            menu_layout(),
            &[ child ]
        )?;

        Ok(self.node_id)
    }

    fn input(&mut self, event: &InputEvent, shell: &mut InputShell<'_>) {
        self.node.input(event, shell);
    }
    fn update(&mut self, shell: &mut UpdateShell<'_>, actions: &mut ActionQueue) {
        self.node.update(shell, actions)
    }
    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.node.draw(shell)
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        self.node.handle_message(message, values, actions).await
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect,
    ) {
        self.node.handle_event(event, event_value, values).await
    }

    async fn reload_skin(
        &mut self, 
        shell: &mut UpdateShell,
    ) {
        self.node.reload_skin(shell).await
    }

}




// TODO: change to spans once implemented so input and output can be color coded (and errors can be red, etc)
fn parse_line(_owner: MessageOwner) -> TextInputAction {
    TextInputAction::Multi(vec![
        TextInputAction::ReflectCallback(Box::new( move |_, r| {
            r.reflect_get_mut::<String>(INPUT_PATH)
            .map(|s| s.clear())
            .inspect_err(|e| warn!("{e:?}"))
            .nope()
        })),

        TextInputAction::ReflectCallback(Box::new(move |s, r| 
            r.reflect_get_mut::<Vec<String>>(OUTPUT_PATH)
            .map(|list| list.push(s.to_string()))
            .inspect_err(|e| warn!("{e:?}"))
            .nope()
        )),
        
        TextInputAction::ReflectCallback(Box::new(move |s, r| {
            let output = match BuildableCalc::parse(s) {
                Ok(cec) => match cec.resolve(r) {
                    Ok(s) => s.as_string(),
                    Err(e) => format!("{e:?}")
                }
                Err(e) => format!("{e:?}")
            };

            r.reflect_get_mut::<Vec<String>>(OUTPUT_PATH)
                .map(|list| list.push(format!("-> {}\n", output)))
                .inspect_err(|e| warn!("{e:?}"))
                .nope()
        })),
    ])
}