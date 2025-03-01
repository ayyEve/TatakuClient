use crate::prelude::*;
use crate::prelude::ui::*;

pub struct DebugWidget {
    debug: Box<dyn Widget>,
    color: Color,
}
impl DebugWidget {
    pub fn new(debug: Box<dyn Widget>, color: Color) -> Self {
        Self {
            debug,
            color,
        }
    }
}
#[async_trait]
impl Widget for DebugWidget {
    fn name(&self) -> Cow<'static, str> { format!("{} (debug)", self.debug.name()).into() }
    fn node_id(&self) -> NodeId { self.debug.node_id() }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.debug.layout(shell)
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        self.debug.draw(shell);

        let Some(bounds) = shell.tree.absolute_bounds(self.node_id()) else { return };
        shell.list.push(Rectangle::new_bounds(
            bounds,
            Color::TRANSPARENT_WHITE,
            Some(Border::new(self.color, 3.0))
        ));
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        self.debug.update(shell, actions);
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        self.debug.handle_message(message, values, actions).await;
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect
    ) {
        self.debug.handle_event(event, event_value, values).await
    }

    async fn reload_skin(&mut self, skin_manager: &mut dyn SkinProvider) {
        self.debug.reload_skin(skin_manager).await
    }
}
