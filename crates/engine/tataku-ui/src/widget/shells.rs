use crate::*;
use crate::tree::*;
use crate::style::*;
use crate::widget::*;
use crate::message::*;
use common::reflect::*;

pub struct MessageShell<'a, Action: Send + Sync + 'static> {
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub source: MessageSource,
    pub handled: bool,
}

pub struct InputShell<'a, Action: Send + Sync + 'static> {
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub source: MessageSource,
    pub mouse_pos: Vector2,

    pub event_consumed: bool,
}
impl<Action: Send + Sync + 'static> InputShell<'_, Action> {
    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }
}
pub struct DrawShell<'a, Action: Send + Sync + 'static> {
    pub tree: &'a Tree<Action>,
    pub values: &'a dyn Reflect,
    pub list: &'a mut graphics::RenderableCollection,
    pub general_theme: GeneralUiTheme,

    pub text_layout_contexts: &'a mut TextLayoutContexts,
}

pub struct UpdateShell<'a, Action: Send + Sync + 'static> {
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,

    pub source: MessageSource,
    pub messages: &'a mut Vec<Message>,
    pub actions: &'a mut Vec<Action>,
    pub skin_manager: &'a mut dyn graphics::SkinProvider,

    pub text_layout_contexts: &'a mut TextLayoutContexts,
}


pub struct LayoutShell<'a, 'css: 'a, Action: Send + Sync + 'static> {
    pub tree: &'a mut Tree<Action>,
    pub values: &'a mut dyn Reflect,
    pub source: MessageSource,
    pub ui_scale: f32,
    pub resolver: &'a mut CssResolver<'css>,

    pub text_layout_contexts: &'a mut TextLayoutContexts,
}
impl<Action: Send + Sync + 'static> LayoutShell<'_,'_, Action> {
    pub fn with_context(
        &mut self,
        node: impl HasNodeId,
        f: impl Fn(&mut TreeData)

    ) {
        let ctx = self.tree
            .get_context_mut(node.get_id())
            .expect("no context?");
        f(ctx);
    }
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
