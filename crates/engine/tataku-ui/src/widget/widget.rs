use crate::prelude::*;
use tataku_input::prelude::*;

pub trait Widget<Action: Send + Sync>: Send + Sync {
    fn name(&self) -> CowStr;
    fn node_id(&self) -> NodeId;

    fn all_children(&'_ self) -> WidgetChildren<'_, Action> { self.children() }
    fn all_children_mut(&'_ mut self) -> WidgetChildrenMut<'_, Action> { self.children_mut() }

    /// helper for default actions
    fn children(&'_ self) -> WidgetChildren<'_, Action> { WidgetChildren::None }
    /// helper for default actions
    fn children_mut(&'_ mut self) -> WidgetChildrenMut<'_, Action> { WidgetChildrenMut::None }

    fn get_style_str(&self) -> ArcStr { ArcStr::default() }
    
    fn layout(&mut self, shell: &mut LayoutShell<Action>) -> taffy::TaffyResult<NodeId>;
    fn init_style(&mut self, shell: &mut LayoutShell<Action>) {
        for i in self.all_children_mut() {
            i.init_style(shell);
        }
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<Action>,
    ) {
        for i in self.children_mut() {
            if shell.event_consumed { return }
            i.input(event, shell);
        }
    }

    fn operation(
        &mut self, 
        operation: &UiOperation, 
        tree: &mut Tree<Action>,
    ) {
        for i in self.children_mut() {
            i.operation(operation, tree);
        }
    }

    fn draw(&self, shell: &mut DrawShell<Action>) {
        for i in self.children() {
            i.draw(shell);
        }
    }
    fn draw_overlay(&self, shell: &mut DrawShell<Action>) {
        for i in self.children() {
            i.draw_overlay(shell);
        }
    }

    fn update(&mut self, shell: &mut UpdateShell<Action>) {
        for i in self.children_mut() {
            i.update(shell);
        }
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell<Action>,
    ) {
        for i in self.children_mut() {
            if shell.handled { return }
            i.handle_message(message, shell);
        }
    }

    fn handle_event(
        &mut self, 
        event: &TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell<Action>,
    ) {
        for i in self.children_mut() {
            if shell.handled { return }
            i.handle_event(event, event_value, shell);
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<Action>) {
        for i in self.all_children_mut() {
            i.reload_skin(shell);
        }
    }

    fn boxed(self) -> Box<dyn Widget<Action>> where Self:Sized + 'static {
        Box::new(self)
    }
}
