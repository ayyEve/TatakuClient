#![allow(unused, dead_code)]

use crate::prelude::*;
use crate::prelude::ui::*;
const Y_PADDING:f32 = 5.0;
const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 30.0);

// pub type ClickFn = Box<dyn Fn(&mut GenericDialog, &mut Game) + Send + Sync>;
pub type ClickFn = Arc<dyn Fn(&mut GenericDialog, &mut ActionQueue) -> Option<TatakuAction> + Send + Sync>;

pub struct GenericDialog {
    button_actions: HashMap<String, ClickFn>,
    node: Box<dyn Widget>,
    node_id: NodeId
}
impl GenericDialog {
    pub fn new(_title: impl AsRef<str>) -> Self {
        Self {
            button_actions: HashMap::new(),
            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE,
        }
    }

    pub fn add_button(&mut self, text: impl ToString, on_click: ClickFn) {
        let text = text.to_string();
        self.button_actions.insert(text, on_click);
    }

    
    fn build_view(&self, owner: MessageOwner) -> Box<dyn Widget> {
        let buttons = self.button_actions.keys()
            .map(|s| Button::new(
                TextWidget::new(s.clone()).boxed()
            )
            .on_press(Message::new(owner, s, MessageValue::Click))
            .boxed()
        );

        col!(
            buttons.collect::<Vec<_>>(),
            height = Dimension::Percent(1.0)
        )
    }
}
impl Widget for GenericDialog {
    fn name(&self) -> Cow<'static, str> { "generic_dialog".into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.node.update_styles(tree, resolver, display_override);
    }
    
    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.build_view(shell.owner);
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(Style::default(), &[child])?;
        Ok(self.node_id)
    }
    
    fn draw(
        &self, 
        shell: &mut DrawShell<'_>, 
    ) {
        
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        actions: &mut ActionQueue
    ) {
        let Some(tag) = message.tag.as_string() else { return }; 

        if let Some(action) = self.button_actions.get(tag).cloned() {
            if let Some(action2) = (action)(self, actions) {
                actions.push(action2)
            }
        }
    }

}
