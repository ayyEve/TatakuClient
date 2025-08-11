use crate::prelude::*;
use crate::prelude::ui::*;

/// Literally an empty element
#[derive(Default)]
pub struct EmptyWidget(pub NodeId);
impl EmptyWidget {
    pub fn new_boxed() -> Box<dyn Widget> {
        Box::new(Self(EMPTY_NODE))
    }
}
impl Widget for EmptyWidget {
    fn name(&self) -> CowStr { "empty_widget".into() }
    fn node_id(&self) -> NodeId { self.0 }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        self.0 = shell.tree.new_leaf()?;
        shell.tree.set_display(self.0, Some(DisplayType::None));
        
        Ok(self.0)
    }
}
