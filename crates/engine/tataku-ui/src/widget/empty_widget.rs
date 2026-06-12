use crate::*;
use crate::tree::*;
use crate::style::*;
use crate::widget::*;

/// Literally an empty element
pub struct EmptyWidget(pub NodeId);
impl EmptyWidget {
    pub fn new_boxed<Action: Send + Sync + 'static>() -> Box<dyn Widget<Action>> {
        Box::new(Self(EMPTY_NODE))
    }
}
impl Default for EmptyWidget {
    fn default() -> Self { Self(EMPTY_NODE) }
}

impl<Action: Send + Sync + 'static> Widget<Action> for EmptyWidget {
    fn name(&self) -> CowStr { "empty_widget".into() }
    fn node_id(&self) -> NodeId { self.0 }

    fn layout(&mut self, shell: &mut LayoutShell<Action>) -> taffy::TaffyResult<NodeId> {
        self.0 = shell.tree.new_leaf()?;
        shell.tree.set_overrides(self.0, |s| s.set_property(css::StyleProperty::Display(DisplayType::None.into())));
        
        Ok(self.0)
    }
}
