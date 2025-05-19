use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct ConditionalWidget {
    if_true: Box<dyn Widget>,
    if_false: Option<Box<dyn Widget>>,
    cond: BuildableCondition,

    #[chain] style: Style,

    value: bool,
    node_id: NodeId,
}
impl ConditionalWidget {
    pub fn new(
        if_true: Box<dyn Widget>,
        if_false: Option<Box<dyn Widget>>,
        mut cond: BuildableCondition,
    ) -> Self {
        // make sure the condition is built
        cond.build();

        Self {
            if_true,
            if_false,
            cond,
            style: Style::default(),

            value: false,

            node_id: EMPTY_NODE
        }
    }

    #[allow(clippy::borrowed_box)] // Box<dyn Widget> doesnt implement dyn Widget, and dereferencing and re-referencing is unecessary and ugly
    fn get_ele(&self) -> Option<&Box<dyn Widget>> {
        if self.value {
            Some(&self.if_true)
        } else {
            self.if_false.as_ref()
        }
    }

    fn get_ele_mut(&mut self) -> Option<&mut Box<dyn Widget>> {
        if self.value {
            Some(&mut self.if_true)
        } else {
            self.if_false.as_mut()
        }
    }
}
impl Widget for ConditionalWidget {
    fn name(&self) -> Cow<'static, str>  { "conditional_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell, 
        _display_override: Option<ui::Display>
    ) {
        self.if_true.update_styles(
            shell, 
            (!self.value).then_some(ui::Display::None)
        );
        
        if let Some(if_false) = &mut self.if_false {
            if_false.update_styles(
                shell, 
                self.value.then_some(ui::Display::None)
            );
        }
    }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>  {
        let mut children = Vec::with_capacity(2);
        children.push(self.if_true.layout(shell)?);
        if let Some(if_false) = self.if_false.as_mut() {
            children.push(if_false.layout(shell)?)
        }

        self.node_id = shell.tree.new_with_children(
            self.style.clone(), 
            &children
        )?;

        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(child) = self.get_ele() else { return };
        child.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        let Some(child) = self.get_ele() else { return };
        child.draw(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        if let Some(child) = self.get_ele_mut() {
            child.input(event, shell);
        }
    }



    fn update(&mut self, shell: &mut UpdateShell) {
        match self.cond.resolve(shell.values) {
            BuildableConditionResult::Error(e) => {
                error!("\n!!!!!!!\nerror with cond {:?}\n{e:?}\n!!!!!!!", self.cond);
                self.cond = BuildableCondition::Failed;
                return;
            }
            BuildableConditionResult::True if !self.value => {
                self.value = true;
                if let Some(child) = self.if_false.as_ref() {
                    shell.actions.push(UiAction::new(
                        child.node_id(), 
                        UiActionType::UpdateDisplay(ui::Display::None)
                    ));
                }

                shell.actions.push(UiAction::new(
                    self.if_true.node_id(), 
                    UiActionType::UpdateDisplay(ui::Display::Flex)
                ));
                shell.actions.push(UiAction::new(
                    self.node_id, 
                    UiActionType::Refresh
                ));
            }

            BuildableConditionResult::False if self.value => {
                self.value = false;

                if let Some(child) = self.if_false.as_ref() {
                    shell.actions.push(UiAction::new(
                        child.node_id(), 
                        UiActionType::UpdateDisplay(ui::Display::Flex)
                    ));
                }

                shell.actions.push(UiAction::new(
                    self.if_true.node_id(), 
                    UiActionType::UpdateDisplay(ui::Display::None)
                ));
                shell.actions.push(UiAction::new(
                    self.node_id, 
                    UiActionType::Refresh
                ));
            }

            _ => {}
        }

        
        if let Some(child) = self.get_ele_mut() { 
            child.update(shell);
        }
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        if let Some(child) = self.get_ele_mut() { 
            child.handle_message(message, shell);
        }
    }

    fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        if let Some(child) = self.get_ele_mut() { 
            child.handle_event(event, event_value, shell);
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.if_true.reload_skin(shell);
        if let Some(if_false) = self.if_false.as_mut() {
            if_false.reload_skin(shell);
        }
    }
}
