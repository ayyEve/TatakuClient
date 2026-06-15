use crate::prelude::*;
use input::InputEvent;
use common::reflect::*;
use tataku::TatakuValue;
use tataku_engine::VariablePathResolver;

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

    #[chain] value: SwitchWidgetValue,

    index: Option<usize>,
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
            value: SwitchWidgetValue::None,

            index: None,
            node_id: ui::EMPTY_NODE
        }
    }

    #[allow(clippy::borrowed_box, reason = "signature")]
    fn get_ele(&self) -> Option<&dyn Widget<actions::Action>> {
        match self.index {
            Some(index) => Some(&*self.cases.get(index)?.widget),
            None => self.default_case.as_deref(),
        }
    }

    fn get_ele_mut(&mut self) -> Option<&mut dyn Widget<actions::Action>> {
        match self.index {
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

        // might be nice to have in the future so i included it here
        // for now its always None
        passed_in: Option<&TatakuValue>,
    ) {
        let switch_value = self.value.resolve(values, passed_in);

        self.index = self
            .cases
            .iter()
            .position(|i|
                match i.cond.resolve(values, switch_value.as_deref(), passed_in) {
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
            shell.tree.set_overrides(
                i.widget.node_id(),
                |s| s.set_property(css::StyleProperty::Display(DisplayType::None.into()))
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
        let previous_index = self.index;
        self.update_value(shell.values, None);

        if self.index != previous_index {
            if let Some(child) = previous_index
                .and_then(|i| self.cases.get(i))
            {
                shell.tree.set_overrides(
                    child.widget.node_id(), 
                    |s| s.set_property(css::StyleProperty::Display(DisplayType::None.into())),
                );
            } else if let Some(default) = &self.default_case {
                shell.tree.set_overrides(
                    default.node_id(), 
                    |s| s.set_property(css::StyleProperty::Display(DisplayType::None.into())),
                );
            }

            if let Some(child) = self.index
                .and_then(|i| self.cases.get(i))
            {
                shell.tree.set_overrides(child.widget.node_id(), |s| s.remove_property(css::CssProperty::Display));
            } else if let Some(default) = &self.default_case {
                shell.tree.set_overrides(default.node_id(), |s| s.remove_property(css::CssProperty::Display));
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


#[derive(Default, From)]
pub enum SwitchWidgetValue {
    #[default]
    None,
    Value(BuildableValue),
    Enum(VariablePathResolver),
}
impl SwitchWidgetValue {
    pub fn resolve<'a>(&'a self, values: &'a dyn Reflect, passed_in: Option<&'a TatakuValue>) -> Option<Cow<'a, TatakuValue>> {
        match self {
            Self::None => None,
            Self::Value(v) => v.resolve(values, passed_in),
            Self::Enum(e) => {
                let path = e.resolve_path(values)
                    .inspect_err(|err| warn!("Failed to resolve path '{e}': {err}"))
                    .ok()?;

                values.reflect_display(&path, None).ok()
                    .map(tataku::TatakuValue::String)
                    .map(Cow::Owned)
            }
        }
    }
}

pub struct SwitchWidgetCase {
    pub cond: SwitchWidgetCaseCond,
    pub widget: Box<dyn Widget<actions::Action>>,
}
impl SwitchWidgetCase {
    pub fn new(
        cond: SwitchWidgetCaseCond,
        widget: Box<dyn Widget<actions::Action>>
    ) -> Self {
        Self {
            cond,
            widget,
        }
    }
}

#[derive(From)]
pub enum SwitchWidgetCaseCond {
    Value(BuildableValue),
    Cond(BuildableCondition),
}
impl SwitchWidgetCaseCond {
    pub fn build(&mut self) {
        match self {
            Self::Cond(c) => c.build(),
            Self::Value(v) => v.build(),
        }
    }

    pub fn resolve(
        &self, 
        values: &dyn Reflect, 
        switch_value: Option<&TatakuValue>,

        passed_in: Option<&TatakuValue>,
    ) -> BuildableConditionResult<'_> {
        // too long otherwise
        type Bcr<'a> = BuildableConditionResult<'a>;

        match self {
            Self::Cond(c) => c.resolve(values),
            Self::Value(v) => {
                let Some(switch_val) = switch_value 
                else { return Bcr::Unbuilt("no switch value") };

                let Some(our_val) = v.resolve(values, passed_in)
                else { return Bcr::Failed };

                Bcr::from(&*our_val == switch_val)
            }
        }
    }
}
