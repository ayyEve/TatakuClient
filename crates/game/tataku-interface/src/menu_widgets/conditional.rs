use crate::prelude::*;
use ui::{
    tree::*,
    style::*,
    widget::*,
};

#[derive(ChainableInitializer)]
pub struct ConditionalWidget {
    if_true: Box<dyn Widget<actions::Action>>,
    if_false: Option<Box<dyn Widget<actions::Action>>>,
    cond: BuildableCondition,

    value: bool,
    node_id: NodeId,
}
impl ConditionalWidget {
    pub fn new(
        if_true: Box<dyn Widget<actions::Action>>,
        if_false: Option<Box<dyn Widget<actions::Action>>>,
        mut cond: BuildableCondition,
    ) -> Self {
        // make sure the condition is built
        cond.build();

        Self {
            if_true,
            if_false,
            cond,

            value: false,
            node_id: ui::EMPTY_NODE
        }
    }

    fn get_ele(&self) -> Option<&dyn Widget<actions::Action>> {
        if self.value {
            Some(&*self.if_true)
        } else {
            self.if_false.as_deref()
        }
    }

    fn get_ele_mut(&mut self) -> Option<&mut dyn Widget<actions::Action>> {
        if self.value {
            Some(&mut *self.if_true)
        } else {
            fn reborrow(w: &mut Box<dyn Widget<actions::Action>>) -> &mut dyn Widget<actions::Action> {
                &mut **w
            }

            self.if_false.as_mut()
                .map(reborrow)
        }
    }
}
impl Widget<actions::Action> for ConditionalWidget {
    fn name(&self) -> CowStr  { "conditional_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        let Some(child) = self.get_ele() 
        else { return WidgetChildren::None };
        
        WidgetChildren::Single(child)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        let Some(child) = self.get_ele_mut() 
        else { return WidgetChildrenMut::None };
        
        WidgetChildrenMut::Single(child)
    }
    fn all_children(&self) -> WidgetChildren<'_, actions::Action> {
        let mut list = Vec::with_capacity(2);
        list.push(&*self.if_true);
        if let Some(f) = &self.if_false {
            list.push(&**f);
        }

        WidgetChildren::OwnedList(list)
    }
    fn all_children_mut<'a>(&'a mut self) -> WidgetChildrenMut<'a, actions::Action> {
        let mut list: Vec<&'a mut dyn Widget<actions::Action>> = Vec::with_capacity(2);
        list.push(&mut *self.if_true);
        if let Some(f) = &mut self.if_false {
            list.push(&mut **f);
        }

        WidgetChildrenMut::OwnedList(list)
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId>  {
        let mut children = Vec::with_capacity(2);
        children.push(self.if_true.layout(shell)?);
        if let Some(if_false) = self.if_false.as_mut() {
            children.push(if_false.layout(shell)?);
        }

        self.node_id = shell.tree.new_with_children(&children)?;
        Ok(self.node_id)
    }
    
    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        for i in self.all_children_mut() {
            i.init_style(shell);
        }

        // set the true condition widget to DisplayType::None so its hidden
        // do not do this for the false widget because if it exists it should be visible by default
        shell.tree.set_overrides(self.if_true.node_id(), |s| s.display = Some(DisplayType::None));
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        match self.cond.resolve(shell.values) {
            BuildableConditionResult::Error(e) => {
                error!("\n!!!!!!!\nerror with cond {:?}\n{e:?}\n!!!!!!!", self.cond);
                self.cond = BuildableCondition::Failed;
                return;
            }
            BuildableConditionResult::True if !self.value => {
                self.value = true;
                if let Some(child) = self.if_false.as_ref() {
                    shell.tree.set_overrides(
                        child.node_id(), 
                        |s| s.display = Some(DisplayType::None),
                    );
                }

                shell.tree.set_overrides(
                    self.if_true.node_id(), 
                    |s| s.display = None,
                );
                shell.tree.mark_for_relayout();
            }

            BuildableConditionResult::False if self.value => {
                self.value = false;

                if let Some(child) = self.if_false.as_ref() {
                    shell.tree.set_overrides(
                        child.node_id(), 
                        |s| s.display = None,
                    );
                }
                
                shell.tree.set_overrides(
                    self.if_true.node_id(), 
                    |s| s.display = Some(DisplayType::None),
                );
            }

            _ => {}
        }

        if let Some(child) = self.get_ele_mut() { 
            child.update(shell);
        }
    }
    
    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
        self.if_true.reload_skin(shell);
        if let Some(if_false) = self.if_false.as_mut() {
            if_false.reload_skin(shell);
        }
    }
}
