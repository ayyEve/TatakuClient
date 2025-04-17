use crate::prelude::*;
use crate::prelude::ui::*;

// TODO: what is this even used for? lmao
#[derive(Default)]
pub struct SpectatorMenu {
    node_id: NodeId
}
impl SpectatorMenu {
    pub fn new() -> Self {
        Self {
            node_id: EMPTY_NODE,
        }
    }
}

// FIXME: all this
impl Widget for SpectatorMenu {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed("spectator_menu") }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf(Style::DEFAULT)?;
        Ok(self.node_id)
    }
    fn draw(&self, _shell: &mut DrawShell<'_>) {}
}
