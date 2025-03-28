use crate::prelude::*;
use crate::prelude::ui::*;

/// Literally an empty element
pub struct EmptyWidget(pub NodeId);
impl EmptyWidget {
    pub fn new_boxed() -> Box<dyn Widget> {
        Box::new(Self(EMPTY_NODE))
    }
}
impl Widget for EmptyWidget {
    fn name(&self) -> Cow<'static, str> { "empty_widget".into() }
    fn node_id(&self) -> NodeId { self.0 }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.0 = shell.tree.new_leaf(Style {
            display: ui::Display::None,
            .. Default::default()
        })?;
        
        Ok(self.0)
    }
}
