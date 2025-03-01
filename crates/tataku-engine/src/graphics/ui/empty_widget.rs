use crate::prelude::*;
use crate::prelude::ui::*;

/// literally an empty element
pub struct EmptyWidget(pub NodeId);
impl EmptyWidget {
    pub fn new() -> Self {
        Self(EMPTY_NODE)
    }
    pub fn new_boxed() -> Box<dyn Widget> {
        Box::new(Self::new())
    }
}

impl Widget for EmptyWidget {
    fn name(&self) -> Cow<'static, str> { "empty".into() }
    fn node_id(&self) -> NodeId { self.0 }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.0 = shell.tree.new_leaf(Style {
            display: ui::Display::None,
            .. Default::default()
        })?;
        
        Ok(self.0)
    }
}
