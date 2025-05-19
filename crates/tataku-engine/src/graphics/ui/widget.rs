use crate::prelude::*;
use crate::prelude::ui::*;

pub trait Widget: Send + Sync {
    fn name(&self) -> Cow<'static, str>;
    fn node_id(&self) -> NodeId;

    fn get_style_str(&self) -> String { String::new() }
    fn set_text_style(&mut self, _style: TextStyle) {}
    fn update_styles(
        &mut self, 
        _shell: &mut StyleShell,
        _display_override: Option<ui::Display>
    ) {}

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>;


    fn input(
        &mut self, 
        _event: &InputEvent, 
        _shell: &mut InputShell,
    ) {}

    fn operation(
        &mut self, 
        _operation: &UiOperation, 
        _tree: &mut Tree
    ) {}

    fn draw(&self, _shell: &mut DrawShell) {}
    fn draw_overlay(&self, _shell: &mut DrawShell) {}

    fn update(&mut self, _shell: &mut UpdateShell) {}
    
    fn handle_message(
        &mut self, 
        _message: &Message, 
        _shell: &mut MessageShell
    ) {}

    fn handle_event(
        &mut self, 
        _event: TatakuEventType, 
        _event_value: Option<TatakuValue>, 
        _shell: &mut MessageShell
    ) {}

    fn reload_skin(&mut self, _shell: &mut UpdateShell) {}

    fn boxed(self) -> Box<dyn Widget> where Self:Sized + 'static {
        Box::new(self)
    }
}


pub trait HasNodeId {
    fn get_id(&self) -> TaffyNodeId;
}
impl<T: Widget> HasNodeId for &T {
    fn get_id(&self) -> TaffyNodeId {
        self.node_id().node_id
    }
}
impl<T: Widget> HasNodeId for &mut T {
    fn get_id(&self) -> TaffyNodeId {
        self.node_id().node_id
    }
}
impl HasNodeId for NodeId {
    fn get_id(&self) -> TaffyNodeId {
        self.node_id
    }
}
impl HasNodeId for TaffyNodeId {
    fn get_id(&self) -> TaffyNodeId {
        *self
    }
}


/// Literally an empty element
#[derive(Default)]
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
