use crate::prelude::*;

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
impl Widget<TatakuAction> for VisualizationWidget {
    fn name(&self) -> CowStr { "visualization".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self, 
        shell: &mut LayoutShell<TatakuAction>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id)
        else { return };

        self.vis.update(bounds, shell.actions);
    }

    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        self.vis.draw(shell.list);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        debug!("reloading vis skin");
        self.vis.reload_skin(shell.skin_manager);
    }
}