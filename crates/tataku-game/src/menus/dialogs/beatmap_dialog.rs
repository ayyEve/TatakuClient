use crate::prelude::*;
use crate::prelude::ui::*;

pub struct BeatmapDialog {
    target_map: Md5Hash,

    node_id: NodeId,
    node: Box<dyn Widget>,
}
impl BeatmapDialog {
    pub fn new(target_map: Md5Hash) -> Self {
        Self {
            target_map,

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE,
        }
    }
}

#[async_trait]
impl Widget for BeatmapDialog {
    fn name(&self) -> Cow<'static, str> { "beatmap_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = col!(
            // delete map
            Button::new(TextWidget::new("Delete Map").boxed()).on_press(Message::new(shell.owner, "delete", MessageValue::Click)).boxed(),

            // copy_hash
            Button::new(TextWidget::new("Copy Hash").boxed()).on_press(Message::new(shell.owner, "copy_hash", MessageValue::Click)).boxed();

            height = Dimension::Percent(1.0)
        );

        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::default(), 
            &[ child ]
        )?;
        Ok(self.node_id)
    }
    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<'_>,
    ) {
        self.node.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.node.draw(shell)
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        actions: &mut ActionQueue,
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &**tag {
            "delete" => {
                actions.push(BeatmapAction::Delete(self.target_map));
                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }

            "copy_hash" => {
                trace!("copy hash map {}", self.target_map);
                match GameWindow::set_clipboard(self.target_map.to_string()) {
                    Ok(_) => actions.push(Notification::default().text("Hash copied to clipboard!").duration(3000.0).color(Color::LIGHT_BLUE)),
                    Err(e) => actions.push(Notification::new_error("Failed to copy hash to clipboard", e)),
                }

                actions.push(UiAction::new(self.node_id, DialogAction::Close));
            }

            _ => {}
        }
    }
}