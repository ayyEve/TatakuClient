use crate::prelude::*;
use crate::prelude::ui::*;

pub struct PauseMenu {
    actions: ActionQueue,
    is_fail_menu: bool,

    bg: Option<Image>,

    node: Box<dyn Widget>,
    node_id: NodeId,
}
impl PauseMenu {
    pub fn new(is_fail_menu: bool) -> PauseMenu {
        PauseMenu {
            actions: ActionQueue::new(),
            // manager: Some(manager),
            is_fail_menu,
            bg: None,

            node: EmptyWidget::new_boxed(),
            node_id: EMPTY_NODE
        }
    }

    pub fn unpause(&mut self, restart: bool) {
        if restart {
            self.actions.push(CurrentGameAction::Restart);
        } else {
            self.actions.push(CurrentGameAction::Resume);
        }
    }

    async fn exit(&mut self) {
        // let Some(manager) = self.manager.take() else { return };
        self.actions.push(CurrentGameAction::Free);
        self.actions.push(MenuAction::set_menu("beatmap_select"));
    }

    fn build_view(&self) -> Box<dyn Widget> {
        let resume_button: Box<dyn Widget> = if !self.is_fail_menu { 
            Button::new(TextWidget::new("Resume").boxed())
                .on_press(Message::new(MessageOwner::Menu, "resume", MessageValue::Click))
                .boxed()
        } else { 
            EmptyWidget::new_boxed()
        };

        let menu = row!(
            // left space so the middle column is centered with a width of 1/3 the total width
            Space::new(FILL, FILL).boxed(),

            // actual items
            col!(
                resume_button,
                Button::new(TextWidget::new("Retry").boxed()).on_press(Message::new(MessageOwner::Menu, "retry", MessageValue::Click)).boxed(),
                Button::new(TextWidget::new("Quit").boxed()).on_press(Message::new(MessageOwner::Menu, "quit", MessageValue::Click)).boxed()
                ;
                
                width = FILL,
                height = FILL,
                spacing = LengthPercentage::Length(10.0)
            ),

            // right space so the middle column is centered with a width of 1/3 the total width
            Space::new(FILL, FILL).boxed();

            width = FILL,
            height = FILL
        );

        // this handles the background image
        ContentBackground::new(menu)
            .width(FILL)
            .height(FILL)
            .image(self.bg.clone())
            .boxed()  
    }
}

#[async_trait]
impl Widget for PauseMenu {
    fn name(&self) -> Cow<'static, str>{ if self.is_fail_menu {"fail_menu"} else {"pause_menu"}.into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.build_view();
        
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(menu_layout(), &[child])?;

        Ok(self.node_id)
    }
    
    async fn handle_message(
        &mut self, 
        message: &Message, 
        _values: &mut dyn Reflect,
        _actions: &mut ActionQueue
    ) {
        info!("got message {message:?}");
        let Some(tag) = message.tag.as_string() else { return };

        match &**tag {
            "resume" => self.unpause(false),
            "retry" => self.unpause(true),
            "quit" => self.exit().await,
            _ => {}
        }
    }
    
    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect
    ) {
        self.node.handle_event(event, event_value, values).await
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<'_>
    ) {
        self.node.input(event, shell);
    }

    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>,
        actions: &mut ActionQueue
    )  {
        self.node.update(shell, actions);
        actions.extend(self.actions.take());
    }
    

    async fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.node.reload_skin(shell).await;

        if self.is_fail_menu {
            self.bg = shell.skin_manager.get_texture("fail-background", &TextureSource::Skin, SkinUsage::Game, false).await
        } else {
            self.bg = shell.skin_manager.get_texture("pause-overlay", &TextureSource::Skin, SkinUsage::Game, false).await
        }

        // FIXME: need to reimplement
        // if let Some(bg) = &mut self.bg {
        //     bg.fit_to_bg_size(WindowSize::get().0, true);
        // }
    }

    fn draw(
        &self, 
        shell: &mut DrawShell<'_>,
    ) {
        self.node.draw(shell);
    }
}
