use crate::prelude::*;
use crate::prelude::ui::*;

// TODO: remove this and replace with conditional element once we can parse multiple function arguments in the AST
pub struct FilterableWidget {
    text_to_check: String,
    variable_to_check: String,

    visible: bool,
    node: Box<dyn Widget>,

    node_id: NodeId,
}
impl FilterableWidget {
    pub fn new(
        node: Box<dyn Widget>,
        text_to_check: String,
        variable_to_check: String,
    ) -> Self {
        Self {
            text_to_check,
            variable_to_check,
            
            visible: true,

            node,
            node_id: EMPTY_NODE
        }
    }
}

impl Widget for FilterableWidget {
    fn name(&self) -> Cow<'static, str>  { "filterable_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell,
        _display_override: Option<ui::Display>
    ) {
        self.node.update_styles(shell, None);
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>  {
        let mut children = Vec::with_capacity(2);
        children.push(self.node.layout(shell)?);
        self.node_id = shell.tree.new_with_children(
            Style::DEFAULT, 
            &children
        )?;

        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell) {
        if !self.visible { return }
        self.node.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        if !self.visible { return }
        self.node.draw_overlay(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        if !self.visible { return }
        self.node.input(event, shell);
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        let Ok(filter) = shell.values.reflect_get::<ItemFilter>(&self.variable_to_check)
            .inspect_err(|e| println!("{e:?}")) 
            else { return };
        let new_visible = filter.check(&self.text_to_check);

        if self.visible != new_visible {
            self.visible = new_visible;
            let display = if self.visible {
                ui::Display::Flex
            } else {
                ui::Display::None
            };
            shell.actions.push(UiAction::new(self.node.node_id(), UiActionType::UpdateDisplay(display)));
        }

        if !self.visible { return }
        self.node.update(shell);
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        if !self.visible { return }
        self.node.handle_message(message, shell);
    }

    fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        if !self.visible { return }
        self.node.handle_event(event, event_value, shell);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.node.reload_skin(shell);
    }
}
