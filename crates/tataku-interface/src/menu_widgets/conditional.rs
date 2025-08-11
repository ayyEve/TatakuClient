use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct ConditionalWidget {
    if_true: Box<dyn Widget>,
    if_false: Option<Box<dyn Widget>>,
    cond: BuildableCondition,

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
    fn name(&self) -> CowStr  { "conditional_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren {
        let Some(child) = self.get_ele() 
        else { return WidgetChildren::None };
        
        WidgetChildren::Single(child)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut {
        let Some(child) = self.get_ele_mut() 
        else { return WidgetChildrenMut::None };
        
        WidgetChildrenMut::Single(child)
    }
    fn all_children(&self) -> WidgetChildren {
        let mut list = Vec::with_capacity(2);
        list.push(&self.if_true);
        if let Some(f) = &self.if_false {
            list.push(f);
        }

        WidgetChildren::OwnedList(list)
    }
    fn all_children_mut(&mut self) -> WidgetChildrenMut {
        let mut list = Vec::with_capacity(2);
        list.push(&mut self.if_true);
        if let Some(f) = &mut self.if_false {
            list.push(f);
        }

        WidgetChildrenMut::OwnedList(list)
    }

    // fn update_styles(
    //     &mut self, 
    //     shell: &mut StyleShell, 
    //     _display_override: Option<ui::DisplayType>
    // ) {
    //     self.if_true.update_styles(
    //         shell, 
    //         (!self.value).then_some(ui::DisplayType::None)
    //     );
        
    //     if let Some(if_false) = &mut self.if_false {
    //         if_false.update_styles(
    //             shell, 
    //             self.value.then_some(ui::DisplayType::None)
    //         );
    //     }
    // }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>  {
        let mut children = Vec::with_capacity(2);
        children.push(self.if_true.layout(shell)?);
        if let Some(if_false) = self.if_false.as_mut() {
            children.push(if_false.layout(shell)?);
        }

        self.node_id = shell.tree.new_with_children(&children)?;
        Ok(self.node_id)
    }
    
    fn init_style(&mut self, shell: &mut LayoutShell) {
        self.children_mut()
            .into_iter()
            .for_each(|c| c.init_style(shell));

        // set the true condition widget to DisplayType::None so its hidden
        // do not do this for the false widget because if it exists it should be visible by default
        shell.tree.set_display(self.if_true.node_id(), Some(DisplayType::None));
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
                        UiActionType::OverrideDisplay(Some(DisplayType::None))
                    ));
                }

                shell.actions.push(UiAction::new(
                    self.if_true.node_id(), 
                    UiActionType::OverrideDisplay(None)
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
                        UiActionType::OverrideDisplay(None)
                    ));
                }

                shell.actions.push(UiAction::new(
                    self.if_true.node_id(), 
                    UiActionType::OverrideDisplay(Some(DisplayType::None))
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
    
    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.if_true.reload_skin(shell);
        if let Some(if_false) = self.if_false.as_mut() {
            if_false.reload_skin(shell);
        }
    }
}
