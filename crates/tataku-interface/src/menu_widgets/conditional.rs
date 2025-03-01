use crate::prelude::*;
use crate::prelude::ui::*;


pub struct ConditionalWidget {
    if_true: Box<dyn Widget>,
    if_false: Option<Box<dyn Widget>>,
    cond: ElementCondition,

    value: bool,
    node_id: NodeId,
}
impl ConditionalWidget {
    pub fn new(
        if_true: Box<dyn Widget>,
        if_false: Option<Box<dyn Widget>>,
        mut cond: ElementCondition,
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

#[async_trait]
impl Widget for ConditionalWidget {
    fn name(&self) -> Cow<'static, str>  { "conditional_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
        let mut children = Vec::with_capacity(2);
        children.push(self.if_true.layout(shell)?);
        if let Some(if_false) = self.if_false.as_mut() {
            children.push(if_false.layout(shell)?)
        }

        self.node_id = shell.tree.new_with_children(
            Style::DEFAULT, 
            &children
        )?;

        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell<'_>) {
        let Some(child) = self.get_ele() else { return };

        // let Some(bounds) = shell.tree.relative_bounds(self) else { return };
        // let trans = Matrix::identity().trans(bounds.pos);
        child.draw(shell);
        // shell.with_transform(trans, |shell| {
        // });
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<'_>,
    ) {
        let Some(child) = self.get_ele_mut() else { return };
        child.input(event, shell);
    }



    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue,
    ) {
        match self.cond.resolve(shell.values) {
            ElementResolve::Error(e) => {
                error!("!!!!!!!");
                error!("error with cond {:?}", self.cond);
                error!("{e:?}");
                error!("!!!!!!!");
                self.cond = ElementCondition::Failed;
                return;
            }
            ElementResolve::True if !self.value => {
                self.value = true;
                if let Some(child) = self.if_false.as_ref() {
                    actions.push(UiAction::new(child.node_id(), UiActionType::UpdateDisplay(ui::Display::None)));
                }

                actions.push(UiAction::new(self.if_true.node_id(), UiActionType::UpdateDisplay(ui::Display::Flex)));
                actions.push(UiAction::new(self.node_id, UiActionType::MarkDirty));
            }

            ElementResolve::False if self.value => {
                self.value = false;

                if let Some(child) = self.if_false.as_ref() {
                    actions.push(UiAction::new(child.node_id(), UiActionType::UpdateDisplay(ui::Display::Flex)));
                }

                actions.push(UiAction::new(self.if_true.node_id(), UiActionType::UpdateDisplay(ui::Display::None)));
                actions.push(UiAction::new(self.node_id, UiActionType::MarkDirty));
            }

            _ => {}
        }

        let Some(child) = self.get_ele_mut() else { return };
        child.update(shell, actions);
    }
    
    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        let Some(child) = self.get_ele_mut() else { return };
        child.handle_message(message, values, actions).await;
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect,
    ) {
        let Some(child) = self.get_ele_mut() else { return };
        child.handle_event(event, event_value, values).await
    }

    async fn reload_skin(
        &mut self, 
        skin_manager: &mut dyn SkinProvider,
    ) {
        self.if_true.reload_skin(skin_manager).await;
        if let Some(if_false) = self.if_false.as_mut() {
            if_false.reload_skin(skin_manager).await;
        }
    }
}
