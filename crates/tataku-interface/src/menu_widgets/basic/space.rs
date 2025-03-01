use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Widget)]
pub struct Space {
    style: Style,
    node_id: NodeId,
}
impl Space {
    pub fn new(width: impl Into<Dimension>, height: impl Into<Dimension>) -> Self {
        Self {
            style: Style {
                size: Size {
                    width: width.into(), 
                    height: height.into()
                },
                ..Default::default()
            },

            node_id: EMPTY_NODE,
        }
    }
}

impl Widget for Space {
    fn name(&self) -> Cow<'static, str> { "space_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf(self.style.clone())?;
        Ok(self.node_id)
    }
}