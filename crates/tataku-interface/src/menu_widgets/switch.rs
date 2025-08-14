use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct SwitchWidget {
    cases: Vec<SwitchWidgetCase>,
    default_case: Option<Box<dyn Widget>>,
    // cond: BuildableCondition,

    value: Option<usize>,
    node_id: NodeId,
}
impl SwitchWidget {
    pub fn new(
        mut cases: Vec<SwitchWidgetCase>,
        default_case: Option<Box<dyn Widget>>,
        // mut cond: BuildableCondition,
    ) -> Self {
        // make sure the conditions are built
        for i in cases.iter_mut() {
            i.cond.build();
        }

        Self {
            // cond,
            cases,
            default_case,

            value: None,
            node_id: EMPTY_NODE
        }
    }

    #[allow(clippy::borrowed_box, reason = "signature")]
    fn get_ele(&self) -> Option<&Box<dyn Widget>> {
        let Some(index) = self.value else {
            return self.default_case.as_ref();
        };

        Some(&self.cases.get(index)?.widget)
    }

    fn get_ele_mut(&mut self) -> Option<&mut Box<dyn Widget>> {
        let Some(index) = self.value else {
            return self.default_case.as_mut();
        };

        Some(&mut self.cases.get_mut(index)?.widget)
    }

    fn update_value(
        &mut self,
        values: &dyn Reflect,
    ) {
        self.value = self
            .cases
            .iter()
            .position(|i| 
                match i.cond.resolve(values) {
                    BuildableConditionResult::True => true,
                    BuildableConditionResult::False => false,
                    _ => false,
                }
            );
    }
}
impl Widget for SwitchWidget {
    fn name(&self) -> CowStr { "switch_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren {
        self.get_ele()
            .map(WidgetChildren::Single)
            .unwrap_or_default()
    }
    fn children_mut(&mut self) -> WidgetChildrenMut {
        self.get_ele_mut()
            .map(WidgetChildrenMut::Single)
            .unwrap_or_default()
    }

    fn all_children(&self) -> WidgetChildren {
        let mut list = self.cases
            .iter()
            .map(|a| &a.widget)
            .collect::<Vec<_>>();
        if let Some(default) = &self.default_case {
            list.push(default);
        }

        WidgetChildren::OwnedList(list)
    }
    fn all_children_mut(&mut self) -> WidgetChildrenMut {
        let mut list = self.cases
            .iter_mut()
            .map(|a| &mut a.widget)
            .collect::<Vec<_>>();
        if let Some(default) = &mut self.default_case {
            list.push(default);
        }

        WidgetChildrenMut::OwnedList(list)
    }

    // fn update_styles(
    //     &mut self, 
    //     shell: &mut StyleShell, 
    //     _display_override: Option<DisplayType>,
    // ) {
    //     return;

    //     let mut first_found = false;

    //     for i in self.cases.iter_mut() {
    //         if !first_found 
    //             && i.cond.resolve(shell.values) == BuildableConditionResult::True 
    //         {
    //             first_found = true;
    //             i.widget.update_styles(
    //                 shell, 
    //                 None
    //             );
    //         } else {
    //             i.widget.update_styles(
    //                 shell, 
    //                 Some(DisplayType::None)
    //             );
    //         }
    //     }

    //     if let Some(default) = &mut self.default_case {
    //         default.update_styles(
    //             shell, 
    //             (!first_found).then_some(DisplayType::None)
    //         );
    //     }
    // }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>  {
        let mut children = self
            .cases
            .iter_mut()
            .map(|i| i.widget.layout(shell))
            .collect::<Result<Vec<_>, _>>()?
            ;
        
        if let Some(default_case) = self.default_case.as_mut() {
            children.push(default_case.layout(shell)?);
        }

        self.node_id = shell.tree.new_with_children(&children)?;
        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell) {
        for i in self.all_children_mut() {
            i.init_style(shell);
        }

        // set all cases to DisplayType::None so they're hidden
        // do not do this for the default case because if it exists it should be visible by default
        for i in self.cases.iter() {
            shell.tree.set_display(
                i.widget.node_id(), 
                Some(DisplayType::None)
            );
        }
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(child) = self.get_ele() 
        else { return };

        child.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        let Some(child) = self.get_ele() 
        else { return };

        child.draw_overlay(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        let Some(child) = self.get_ele_mut() 
        else { return };

        child.input(event, shell);
    }



    fn update(&mut self, shell: &mut UpdateShell) {
        let previous_value = self.value;
        self.update_value(shell.values);

        if self.value != previous_value {
            if let Some(child) = previous_value
                .and_then(|i| self.cases.get(i)) 
            {
                shell.actions.push(UiAction::new(
                    child.widget.node_id(), 
                    UiActionType::OverrideDisplay(Some(DisplayType::None))
                ));
            } else if let Some(default) = &self.default_case {
                shell.actions.push(UiAction::new(
                    default.node_id(), 
                    UiActionType::OverrideDisplay(Some(DisplayType::None))
                ));
            }

            if let Some(child) = self.value
                .and_then(|i| self.cases.get(i)) 
            {
                shell.actions.push(UiAction::new(
                    child.widget.node_id(), 
                    UiActionType::OverrideDisplay(None)
                ));
            }  else if let Some(default) = &self.default_case {
                shell.actions.push(UiAction::new(
                    default.node_id(), 
                    UiActionType::OverrideDisplay(None)
                ));
            }
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
        event: &TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        if let Some(child) = self.get_ele_mut() { 
            child.handle_event(event, event_value, shell);
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        for i in self.cases.iter_mut() {
            i.widget.reload_skin(shell);
        }
        if let Some(default) = self.default_case.as_mut() {
            default.reload_skin(shell);
        }
    }
}


pub struct SwitchWidgetCase {
    pub cond: BuildableCondition,
    pub widget: Box<dyn Widget>,
}

