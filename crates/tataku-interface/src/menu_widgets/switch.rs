use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct SwitchWidget {
    cases: Vec<SwitchWidgetCase>,
    default_case: Option<Box<dyn Widget>>,
    // cond: BuildableCondition,

    #[chain] style: Style,

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
            style: Style::default(),

            value: None,
            node_id: EMPTY_NODE
        }
    }

    #[allow(clippy::borrowed_box, reason = "signature")]
    fn get_ele(
        &self,
        _values: &dyn Reflect,
    ) -> Option<&Box<dyn Widget>> {
        let Some(index) = self.value else {
            return self.default_case.as_ref();
        };

        Some(&self.cases.get(index)?.widget)
        // for i in self.cases.iter() {
        //     if i.cond.resolve(values) == BuildableConditionResult::True {
        //         return Some(&i.widget)
        //     }
        // }

        // if let Some(default) = &self.default_case {
        //     Some(default)
        // } else {
        //     None
        // }
    }

    fn get_ele_mut(
        &mut self,
        _values: &dyn Reflect,
    ) -> Option<&mut Box<dyn Widget>> {
        let Some(index) = self.value else {
            return self.default_case.as_mut();
        };

        Some(&mut self.cases.get_mut(index)?.widget)

        // for i in self.cases.iter_mut() {
        //     if i.cond.resolve(values) == BuildableConditionResult::True {
        //         return Some(&mut i.widget)
        //     }
        // }

        // if let Some(default) = &mut self.default_case {
        //     Some(default)
        // } else {
        //     None
        // }
    }

    fn update_value(
        &mut self,
        values: &dyn Reflect,
    ) {
        self.value = self
            .cases
            .iter()
            .enumerate()
            .find(|(_, i)| 
                match i.cond.resolve(values) {
                    BuildableConditionResult::True => true,
                    BuildableConditionResult::False => false,
                    _ => false,
                }
            )
            .map(|(n, _)| n);
    }
}
impl Widget for SwitchWidget {
    fn name(&self) -> CowStr { "switch_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell, 
        _display_override: Option<ui::Display>,
    ) {
        let mut first_found = false;

        for i in self.cases.iter_mut() {
            if !first_found 
                && i.cond.resolve(shell.values) == BuildableConditionResult::True 
            {
                first_found = true;
                i.widget.update_styles(
                    shell, 
                    None
                );
            } else {
                i.widget.update_styles(
                    shell, 
                    Some(ui::Display::None)
                );
            }
        }

        if let Some(default) = &mut self.default_case {
            default.update_styles(
                shell, 
                (!first_found).then_some(ui::Display::None)
            );
        }
    }

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

        self.node_id = shell.tree.new_with_children(
            self.style.clone(), 
            &children
        )?;

        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell) {
        let Some(child) = self.get_ele(shell.values) 
        else { return };

        child.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        let Some(child) = self.get_ele(shell.values) 
        else { return };

        child.draw_overlay(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell,
    ) {
        let Some(child) = self.get_ele_mut(shell.values) 
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
                    UiActionType::UpdateDisplay(ui::Display::None)
                ));
            }

            if let Some(child) = self.value
                .and_then(|i| self.cases.get(i)) 
            {
                shell.actions.push(UiAction::new(
                    child.widget.node_id(), 
                    UiActionType::UpdateDisplay(ui::Display::Flex)
                ));
            }
        }

        
        // match self.cond.resolve(shell.values) {
        //     BuildableConditionResult::Error(e) => {
        //         error!("\n!!!!!!!\nerror with cond {:?}\n{e:?}\n!!!!!!!", self.cond);
        //         self.cond = BuildableCondition::Failed;
        //         return;
        //     }
        //     BuildableConditionResult::True if !self.value => {
        //         self.value = true;
        //         if let Some(child) = self.if_false.as_ref() {
        //             shell.actions.push(UiAction::new(
        //                 child.node_id(), 
        //                 UiActionType::UpdateDisplay(ui::Display::None)
        //             ));
        //         }

        //         shell.actions.push(UiAction::new(
        //             self.if_true.node_id(), 
        //             UiActionType::UpdateDisplay(ui::Display::Flex)
        //         ));
        //         shell.actions.push(UiAction::new(
        //             self.node_id, 
        //             UiActionType::Refresh
        //         ));
        //     }

        //     BuildableConditionResult::False if self.value => {
        //         self.value = false;

        //         if let Some(child) = self.if_false.as_ref() {
        //             shell.actions.push(UiAction::new(
        //                 child.node_id(), 
        //                 UiActionType::UpdateDisplay(ui::Display::Flex)
        //             ));
        //         }

        //         shell.actions.push(UiAction::new(
        //             self.if_true.node_id(), 
        //             UiActionType::UpdateDisplay(ui::Display::None)
        //         ));
        //         shell.actions.push(UiAction::new(
        //             self.node_id, 
        //             UiActionType::Refresh
        //         ));
        //     }

        //     _ => {}
        // }

        if let Some(child) = self.get_ele_mut(shell.values) { 
            child.update(shell);
        }
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        if let Some(child) = self.get_ele_mut(shell.values) { 
            child.handle_message(message, shell);
        }
    }

    fn handle_event(
        &mut self, 
        event: &TatakuEventType, 
        event_value: Option<&TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        if let Some(child) = self.get_ele_mut(shell.values) { 
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

