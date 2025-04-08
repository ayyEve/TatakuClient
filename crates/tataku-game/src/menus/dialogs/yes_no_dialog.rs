use crate::prelude::*;
use crate::prelude::ui::*;
use tokio::sync::mpsc::{ Sender, Receiver, channel };

// TODO: move to Messages and not use a channel
pub struct YesNoDialog {
    title: &'static str,
    prompt: String,

    show_cancel: bool,
    sender: Sender<YesNoResult>,

    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl YesNoDialog {
    pub fn new(title: &'static str, prompt: impl ToString, show_cancel: bool) -> (Receiver<YesNoResult>, Self) {
        let prompt = prompt.to_string();

        // create the sender and receiver to send the result of this dialog
        let (sender, receiver) = channel(1);

        // create the dialog
        (receiver, Self {
            title,
            prompt,
            show_cancel,
            sender,

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        })
    }



    fn view(&self, owner: MessageOwner) -> Box<dyn Widget> {
        col!(
            // prompt
            TextWidget::new(self.prompt.clone()).boxed(),
            row!(
                // yes button
                Button::new(TextWidget::new("Yes").boxed()).on_press(Message::new(owner, "yes", MessageValue::Click)).boxed(),
                // no button
                Button::new(TextWidget::new("No").boxed()).on_press(Message::new(owner, "no", MessageValue::Click)).boxed(),
                // cancel
                self.show_cancel.then(|| 
                    Button::new(
                        TextWidget::new("Cancel").boxed()
                    )
                    .on_press(Message::new(owner, "cancel", MessageValue::Click))
                    .boxed()
                ).unwrap_or_else(|| EmptyWidget::new_boxed())
                ;
                width = FILL
                // spacing = MARGIN
            )
            ;

            width = FILL,
            height = FILL
        )
    }
}

#[async_trait]
impl Widget for YesNoDialog {
    fn name(&self) -> Cow<'static, str> { "yes_no_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.node.update_styles(tree, resolver, display_override);
    }
    
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.view(shell.owner);
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(Style::default(), &[child])?;
        
        Ok(self.node_id)
    }

    fn draw(
        &self, 
        shell: &mut DrawShell<'_>, 
    ) {
        self.node.draw(shell);
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &**tag {
            "force_close" => {
                self.sender.try_send(YesNoResult::Cancel).unwrap();
            }

            "yes" => {
                self.sender.try_send(YesNoResult::Yes).unwrap();
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }
            "no" => {
                self.sender.try_send(YesNoResult::No).unwrap();
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }
            "cancel" => {
                self.sender.try_send(YesNoResult::Cancel).unwrap();
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }

            _ => {}
        }
    }
}

// impl Dialog for YesNoDialog {
//     fn title(&self) -> &'static str { self.title }
//     fn get_num(&self) -> usize { self.num }
//     fn set_num(&mut self, num: usize) { self.num = num }

//     fn should_close(&self) -> bool { self.should_close }
//     // fn get_bounds(&self) -> Bounds {
//     //     Bounds::new(Vector2::ZERO, self.prompt_size + Vector2::with_y(BUTTON_SIZE.y + BUTTON_MARGIN.y * 2.0))
//     // }
    
//     fn force_close(&mut self) { 
//         self.should_close = true; 
//     }
    


//     // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
//     //     // background
//     //     self.draw_background(Color::GRAY, offset, list);

//     //     // prompt text
//     //     let mut text = Text::new(offset, FONT_SIZE, self.prompt.clone(), Color::BLACK, Font::Main);
//     //     text.center_text(&Bounds::new(offset, self.prompt_size));
//     //     list.push(text);

//     //     // buttons
//     //     self.yes_button.draw(offset, list);
//     //     self.no_button.draw(offset, list);
//     //     if let Some(cancel_button) = &mut self.cancel_button {
//     //         cancel_button.draw(offset, list);
//     //     }
//     // }
// }


#[derive(Copy, Clone, Debug)]
pub enum YesNoResult {
    Yes,
    No,
    Cancel
}
