use crate::prelude::*;
use ui::{
    tree::*,
    widget::*,
};

pub struct VisualizationWidget {
    vis: MenuVisualization,
    node_id: NodeId,
}
impl VisualizationWidget {
    pub fn new(vis: MenuVisualization) -> Self {
        Self {
            vis,
            node_id: NodeId::default(),
        }
    }
}
impl Widget<actions::Action> for VisualizationWidget {
    fn name(&self) -> CowStr { "visualization".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self, 
        shell: &mut LayoutShell<actions::Action>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id)
        else { return };

        self.vis.update(bounds, shell.actions);
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        self.vis.draw(shell.list);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
        debug!("reloading vis skin");
        self.vis.reload_skin(shell.skin_manager);
    }
}