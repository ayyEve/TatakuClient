use crate::*;
use crate::tree::*;
use crate::style::*;
use crate::message::*;
use common::reflect::*;

pub struct LayoutShell<'a, 'css: 'a, Action: Send + Sync + 'static> {
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub source: MessageSource,
    pub ui_scale: f32,
    pub resolver: &'a mut CssResolver<'css>,

    // pub default_css: &'a str,
    pub text_layout_contexts: &'a mut TextLayoutContexts,
}
impl<Action: Send + Sync + 'static> LayoutShell<'_,'_, Action> {
    pub fn with_context(
        &mut self,
        node: NodeId,
        f: impl Fn(&mut TreeData)
    ) {
        let ctx = self.tree
            .get_context_mut(node)
            .expect("no context?");
        f(ctx);
    }
}
impl<A: Send + Sync + 'static> HasTree<A> for LayoutShell<'_, '_, A> {
    fn tree(&self) -> &Tree<A> { self.tree }
}
impl<A: Send + Sync + 'static> HasTreeMut<A> for LayoutShell<'_, '_, A> {
    fn tree_mut(&mut self) -> &mut Tree<A> { self.tree }
}

// Input shell
pub struct InputShell<'a, Action: Send + Sync + 'static> {
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub source: MessageSource,
    pub mouse_pos: Vector2,

    pub event_consumed: bool,
}
impl<A: Send + Sync + 'static> HasTree<A> for InputShell<'_, A> {
    fn tree(&self) -> &Tree<A> { self.tree }
}
impl<A: Send + Sync + 'static> HasTreeMut<A> for InputShell<'_, A> {
    fn tree_mut(&mut self) -> &mut Tree<A> { self.tree }
}

// Message shell
pub struct MessageShell<'a, Action: Send + Sync + 'static> {
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub source: MessageSource,
    pub handled: bool,
}
impl<A: Send + Sync + 'static> HasTree<A> for MessageShell<'_, A> {
    fn tree(&self) -> &Tree<A> { self.tree }
}
impl<A: Send + Sync + 'static> HasTreeMut<A> for MessageShell<'_, A> {
    fn tree_mut(&mut self) -> &mut Tree<A> { self.tree }
}


// Update shell
pub struct UpdateShell<'a, Action: Send + Sync + 'static> {
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,

    pub source: MessageSource,
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
    pub skin_manager: &'a mut dyn graphics::SkinProvider,

    pub default_css: &'a str,
    pub text_layout_contexts: &'a mut TextLayoutContexts,
}
impl<A: Send + Sync + 'static> HasTree<A> for UpdateShell<'_, A> {
    fn tree(&self) -> &Tree<A> { self.tree }
}
impl<A: Send + Sync + 'static> HasTreeMut<A> for UpdateShell<'_, A> {
    fn tree_mut(&mut self) -> &mut Tree<A> { self.tree }
}


// Draw shell
pub struct DrawShell<'a, Action: Send + Sync + 'static> {
    pub tree: &'a Tree<Action>,
    pub values: &'a dyn Reflect,
    pub list: &'a mut graphics::RenderableCollection,
    pub general_theme: GeneralUiTheme,

    pub text_layout_contexts: &'a mut TextLayoutContexts,
}
impl<A: Send + Sync + 'static> HasTree<A> for DrawShell<'_, A> {
    fn tree(&self) -> &Tree<A> { self.tree }
}


pub struct GenericShell<'a, Action: Send + Sync +'static> {
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
}
impl<'a, 'b:'a, Action: Send + Sync +'static> From<&'b mut MessageShell<'a, Action>> for GenericShell<'a, Action> {
    fn from(value: &'b mut MessageShell<'a, Action>) -> Self {
        Self {
            tree: value.tree,
            values: value.values,
            messages: value.messages,
            actions: value.actions
        }
    }
}
impl<'a, 'b:'a, Action: Send + Sync +'static> From<&'b mut UpdateShell<'a, Action>> for GenericShell<'a, Action> {
    fn from(value: &'a mut UpdateShell<'b, Action>) -> Self {
        Self {
            tree: value.tree,
            values: value.values,
            messages: value.messages,
            actions: value.actions
        }
    }
}
impl<'a, 'b:'a, Action: Send + Sync +'static> From<&'b mut InputShell<'a, Action>> for GenericShell<'a, Action> {
    fn from(value: &'b mut InputShell<'a, Action>) -> Self {
        Self {
            tree: value.tree,
            values: value.values,
            messages: value.messages,
            actions: value.actions
        }
    }
}

pub struct TextLayoutContexts {
    pub font: parley::FontContext,
    pub layout: parley::LayoutContext<Color>,
}
impl TextLayoutContexts {
    pub fn new() -> Self {
        Self {
            font: parley::FontContext::new(),
            layout: parley::LayoutContext::new(),
        }
    }

    pub fn simple_text(
        &mut self,
        text: &str,
        style: &TextStyle,
    ) -> parley::Layout<Color> {
        let mut builder = self.layout.tree_builder(
            &mut self.font,
            1.0, // gui/dpi scale
            true,
            &style.into(),
        );

        builder.push_text(text);

        let (layout, _text) = builder.build();

        layout
    }
}



pub trait HasTree<A: Send + Sync + 'static> {
    fn tree(&self) -> &Tree<A>;

    fn state(&self, node_id: NodeId) -> Option<ElementState> {
        Some(self.tree().get_context(node_id)?.element_data.state)
    }
    fn with_ctx<T>(&self, node_id: NodeId, f: impl FnOnce(&TreeData) -> T) -> Option<T> {
        let ctx = self.tree().get_context(node_id)?;
        Some(f(ctx))
    } 
}

pub trait HasTreeMut<A: Send + Sync + 'static>: HasTree<A> {
    fn tree_mut(&mut self) -> &mut Tree<A>;

    fn state_mut(&mut self, node_id: NodeId) -> Option<&mut ElementState> {
        Some(&mut self.tree_mut().get_context_mut(node_id)?.element_data.state)
    }

    fn with_ctx_mut<T>(&mut self, node_id: NodeId, f: impl FnOnce(&mut TreeData) -> T) -> Option<T> {
        let ctx = self.tree_mut().get_context_mut(node_id)?;
        Some(f(ctx))
    } 
}

