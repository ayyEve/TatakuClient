use crate::*;
use crate::tree::*;
use crate::widget::*;
use crate::message::*;

pub trait Widget<Action: Send + Sync>: Send + Sync {
    fn name(&self) -> CowStr;
    fn node_id(&self) -> NodeId;

    fn all_children(&self) -> WidgetChildren<'_, Action> { self.children() }
    fn all_children_mut(&mut self) -> WidgetChildrenMut<'_, Action> { self.children_mut() }

    fn children(&self) -> WidgetChildren<'_, Action> { WidgetChildren::None }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, Action> { WidgetChildrenMut::None }

    fn get_style_str(&self) -> ArcStr { ArcStr::default() }
    
    fn layout(&mut self, shell: &mut LayoutShell<Action>) -> taffy::TaffyResult<NodeId>;
    fn init_style(&mut self, shell: &mut LayoutShell<Action>) {
        for i in self.all_children_mut() {
            i.init_style(shell);
        }
    }

    fn input(
        &mut self,
        event: &input::InputEvent,
        shell: &mut InputShell<Action>,
    ) {
        for i in self.children_mut() {
            if shell.event_consumed { return }
            i.input(event, shell);
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
        event: &input::TatakuEvent,
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

impl<Action: Send + Sync> Widget<Action> for Box<dyn Widget<Action>> {
    fn name(&self) -> CowStr {
        Widget::name(&**self)
    }

    fn node_id(&self) -> NodeId {
        Widget::node_id(&**self)
    }

    fn layout(&mut self, shell: &mut LayoutShell<Action>) -> taffy::TaffyResult<NodeId> {
        Widget::layout(&mut **self, shell)
    }

    fn all_children(&self) -> WidgetChildren<'_, Action> {
        Widget::all_children(&**self)
    }

    fn all_children_mut(&mut self) -> WidgetChildrenMut<'_, Action> {
        Widget::all_children_mut(&mut **self)
    }

    fn children(&self) -> WidgetChildren<'_, Action> {
        Widget::children(&**self)
    }

    fn children_mut(&mut self) -> WidgetChildrenMut<'_, Action> {
        Widget::children_mut(&mut **self)
    }

    fn get_style_str(&self) -> ArcStr {
        Widget::get_style_str(&**self)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<Action>) {
        Widget::init_style(&mut **self, shell);
    }

    fn input(
        &mut self,
        event: &tataku_input::InputEvent,
        shell: &mut InputShell<Action>,
    ) {
        Widget::input(&mut **self, event, shell);
    }

    fn draw(&self, shell: &mut DrawShell<Action>) {
        Widget::draw(&**self, shell);
    }

    fn draw_overlay(&self, shell: &mut DrawShell<Action>) {
        Widget::draw_overlay(&**self, shell);
    }

    fn update(&mut self, shell: &mut UpdateShell<Action>) {
        Widget::update(&mut **self, shell);
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<Action>,
    ) {
        Widget::handle_message(&mut **self, message, shell);
    }

    fn handle_event(
        &mut self,
        event: &tataku_input::TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<Action>,
    ) {
        Widget::handle_event(&mut **self, event, event_value, shell);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<Action>) {
        Widget::reload_skin(&mut **self, shell);
    }

    fn boxed(self) -> Box<dyn Widget<Action>> where Self:Sized + 'static {
        self
    }
}
