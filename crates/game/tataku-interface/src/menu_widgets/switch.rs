use crate::prelude::*;
use input::InputEvent;
use common::reflect::*;
use tataku::TatakuValue;
use ui::{
    tree::*,
    style::*,
    widget::*,
    message::*,
};

#[derive(ChainableInitializer)]
pub struct SwitchWidget {
    cases: Vec<SwitchWidgetCase>,
    default_case: Option<Box<dyn Widget<actions::Action>>>,
    // cond: BuildableCondition,

    value: Option<usize>,
    node_id: NodeId,
}
impl SwitchWidget {
    pub fn new(
        mut cases: Vec<SwitchWidgetCase>,
        default_case: Option<Box<dyn Widget<actions::Action>>>,
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
            node_id: ui::EMPTY_NODE
        }
    }

    #[allow(clippy::borrowed_box, reason = "signature")]
    fn get_ele(&self) -> Option<&dyn Widget<actions::Action>> {
        match self.value {
            Some(index) => Some(&*self.cases.get(index)?.widget),
            None => self.default_case.as_deref(),
        }
    }

    fn get_ele_mut(&mut self) -> Option<&mut dyn Widget<actions::Action>> {
        match self.value {
            Some(index) => Some(&mut *self.cases.get_mut(index)?.widget),
            None => {
                fn reborrow(w: &mut Box<dyn Widget<actions::Action>>) -> &mut dyn Widget<actions::Action> {
                    &mut **w
                }

                self.default_case.as_mut()
                    .map(reborrow)
            },
        }
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
impl Widget<actions::Action> for SwitchWidget {
    fn name(&self) -> CowStr { "switch_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        self.get_ele()
            .map(WidgetChildren::Single)
            .unwrap_or_default()
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        self.get_ele_mut()
            .map(WidgetChildrenMut::Single)
            .unwrap_or_default()
    }

    fn all_children(&self) -> WidgetChildren<'_, actions::Action> {
        let mut list = self.cases
            .iter()
            .map(|a| &*a.widget)
            .collect::<Vec<_>>();
        if let Some(default) = self.default_case.as_deref() {
            list.push(default);
        }

        WidgetChildren::OwnedList(list)
    }
    fn all_children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        fn reborrow(w: &mut Box<dyn Widget<actions::Action>>) -> &mut dyn Widget<actions::Action> {
            &mut **w
        }

        let mut list = self.cases
            .iter_mut()
            .map(|a| &mut a.widget)
            .map(reborrow)
            .collect::<Vec<_>>();
        if let Some(default) = self.default_case.as_deref_mut() {
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

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId>  {
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

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
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

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(child) = self.get_ele()
        else { return };

        child.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(child) = self.get_ele()
        else { return };

        child.draw_overlay(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        let Some(child) = self.get_ele_mut()
        else { return };

        child.input(event, shell);
    }



    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let previous_value = self.value;
        self.update_value(shell.values);

        if self.value != previous_value {
            if let Some(child) = previous_value
                .and_then(|i| self.cases.get(i))
            {
                shell.actions.push(actions::ui::UiAction::new(
                    child.widget.node_id(),
                    actions::ui::UiActionType::OverrideDisplay(Some(DisplayType::None))
                ).into());
            } else if let Some(default) = &self.default_case {
                shell.actions.push(actions::ui::UiAction::new(
                    default.node_id(),
                    actions::ui::UiActionType::OverrideDisplay(Some(DisplayType::None))
                ).into());
            }

            if let Some(child) = self.value
                .and_then(|i| self.cases.get(i))
            {
                shell.actions.push(actions::ui::UiAction::new(
                    child.widget.node_id(),
                    actions::ui::UiActionType::OverrideDisplay(None)
                ).into());
            }  else if let Some(default) = &self.default_case {
                shell.actions.push(actions::ui::UiAction::new(
                    default.node_id(),
                    actions::ui::UiActionType::OverrideDisplay(None)
                ).into());
            }
        }

        if let Some(child) = self.get_ele_mut() {
            child.update(shell);
        }
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
    ) {
        if let Some(child) = self.get_ele_mut() {
            child.handle_message(message, shell);
        }
    }

    fn handle_event(
        &mut self,
        event: &input::TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<actions::Action>,
    ) {
        if let Some(child) = self.get_ele_mut() {
            child.handle_event(event, event_value, shell);
        }
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
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
    pub widget: Box<dyn Widget<actions::Action>>,
}
