use crate::prelude::*;
use crate::prelude::ui::*;

pub struct StyleShell<'a, 'css: 'a> {
    pub tree: &'a mut Tree,
    pub values: &'a dyn Reflect,
    pub resolver: &'a mut CssResolver<'css>,
}
impl StyleShell<'_, '_> {
    pub fn style(&self, node: NodeId) -> Option<&CssStyle> {
        Some(
            &self
            .tree
            .get_context(node)?
            .element_data
            .style()
            .0
        )
    }
}

pub struct MessageShell<'a> {
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut ActionQueue,
    pub tree: &'a mut Tree,
    pub values: &'a mut dyn Reflect,
    pub owner: MessageOwner,
    pub handled: bool,
}

pub struct InputShell<'a> {
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut ActionQueue,
    pub tree: &'a mut Tree,
    pub values: &'a mut dyn Reflect,
    pub owner: MessageOwner,
    pub mouse_pos: Vector2,

    pub event_consumed: bool,
}
impl InputShell<'_> {
    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }
}
pub struct DrawShell<'a> {
    pub tree: &'a Tree,
    pub values: &'a dyn Reflect,
    pub list: &'a mut RenderableCollection,
    pub general_theme: GeneralUiTheme,
}

pub struct UpdateShell<'a> {
    pub tree: &'a mut Tree,
    pub values: &'a mut dyn Reflect,

    pub owner: MessageOwner,
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut ActionQueue,
    pub skin_manager: &'a mut dyn SkinProvider
}


pub struct LayoutShell<'a> {
    pub tree: &'a mut Tree,
    pub values: &'a mut dyn Reflect,
    pub owner: MessageOwner,
    pub ui_scale: f32
}
impl LayoutShell<'_> {
    pub fn with_context(&mut self, node: impl HasNodeId, f: impl Fn(&mut TreeData)) { 
        let ctx = self.tree.get_context_mut(node.get_id())
            .expect("no context?");
        f(ctx);
    }
}
