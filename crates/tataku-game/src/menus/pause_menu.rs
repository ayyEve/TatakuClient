use crate::prelude::*;

// const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
// const Y_MARGIN:f32 = 20.0;
// const Y_OFFSET:f32 = 10.0;

pub struct PauseMenu {
    actions: ActionQueue,

    // beatmap: Arc<Mutex<Beatmap>>,
    manager: Option<Box<GameplayManager>>,
    is_fail_menu: bool,

    bg: Option<Image>
}
impl PauseMenu {
    pub async fn new(manager: Box<GameplayManager>, is_fail_menu: bool) -> PauseMenu {
        PauseMenu {
            actions: ActionQueue::new(),
            manager: Some(manager),
            is_fail_menu,
            bg: None
        }
    }

    pub fn unpause(&mut self) {
        let Some(manager) = self.manager.take() else { return };
        self.actions.push(GameAction::ResumeMap(manager));
    }

    async fn retry(&mut self) {
        if let Some(manager) = &mut self.manager {
            manager.reset().await;
        }
        self.unpause();
    }

    async fn exit(&mut self) {
        let Some(manager) = self.manager.take() else { return };
        self.actions.push(GameAction::FreeGameplay(manager));
        self.actions.push(MenuAction::set_menu("beatmap_select"));
    }
}

#[async_trait]
impl AsyncMenu for PauseMenu {
    fn get_name(&self) -> &'static str { if self.is_fail_menu {"fail"} else {"pause"} }
    
    fn view(&self, _values: &mut dyn Reflect) -> IcedElement {
        use crate::prelude::iced_elements::*;

        let resume_button: IcedElement = if !self.is_fail_menu { 
            Button::new(Text::new("Resume")).on_press(Message::new(MessageOwner::Menu, "resume", MessageType::Click)).into() 
        } else { 
            EmptyElement.into() 
        };

        let menu = row!(
            // left space so the middle column is centered with a width of 1/3 the total width
            Space::new(Fill, Fill),

            // actual items
            col!(
                resume_button,
                Button::new(Text::new("Retry")).on_press(Message::new(MessageOwner::Menu, "retry", MessageType::Click)),
                Button::new(Text::new("Quit")).on_press(Message::new(MessageOwner::Menu, "quit", MessageType::Click));
                
                width = Fill,
                height = Fill,
                spacing = 10.0
            ),

            // right space so the middle column is centered with a width of 1/3 the total width
            Space::new(Fill, Fill);

            width = Fill,
            height = Fill
        );

        // this handles the background image
        ContentBackground::new(menu)
            .width(Fill)
            .height(Fill)
            .image(self.bg.clone())

            .into_element()
    }
    
    async fn handle_message(&mut self, message: Message, _values: &mut dyn Reflect) {
        info!("got message {message:?}");
        let Some(tag) = message.tag.as_string() else { return };

        match &*tag {
            "resume" => self.unpause(),
            "retry" => self.retry().await,
            "quit" => self.exit().await,

            _ => {}
        }

    }
    
    async fn update(&mut self, _values: &mut dyn Reflect) -> Vec<TatakuAction> {
        self.actions.take()
    }
    

    async fn reload_skin(&mut self, skin_manager: &mut dyn SkinProvider) {
        if self.is_fail_menu {
            self.bg = skin_manager.get_texture("fail-background", &TextureSource::Skin, SkinUsage::Game, false).await
        } else {
            self.bg = skin_manager.get_texture("pause-overlay", &TextureSource::Skin, SkinUsage::Game, false).await
        }

        if let Some(bg) = &mut self.bg {
            bg.fit_to_bg_size(WindowSize::get().0, true);
        }
    }
    // async fn draw(&mut self, list: &mut RenderableCollection) {
    //     let pos_offset = Vector2::ZERO;

    //     if let Some(bg) = self.bg.clone() {
    //         list.push(bg)
    //     }

    //     // draw buttons
    //     if !self.is_fail_menu {
    //         self.continue_button.draw(pos_offset, list);
    //     }
    //     self.retry_button.draw(pos_offset, list);
    //     self.exit_button.draw(pos_offset, list);
    // }

    // async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
    //     // continue map
    //     if !self.is_fail_menu && self.continue_button.on_click(pos, button, mods) {
    //         self.unpause(game);
    //         return;
    //     }
        
    //     // retry
    //     if self.retry_button.on_click(pos, button, mods) {
    //         self.retry(game).await;
    //         return;
    //     }

    //     // return to song select
    //     if self.exit_button.on_click(pos, button, mods) { self.exit(game).await }
    // }

    // async fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {
    //     self.continue_button.on_mouse_move(pos);
    //     self.retry_button.on_mouse_move(pos);
    //     self.exit_button.on_mouse_move(pos);
    // }

    // async fn on_key_press(&mut self, key:Key, game:&mut Game, _mods:KeyModifiers) {
    //     if key == Key::Escape {
    //         if self.is_fail_menu {
    //             self.exit(game).await;
    //         } else {
    //             self.unpause(game);
    //         }
    //     }
    // }


    // async fn controller_down(&mut self, game:&mut Game, _controller: &GamepadInfo, button: ControllerButton) -> bool {
    //     let max = if self.is_fail_menu {2} else {3};

    //     let mut changed = false;

    //     match button {
    //         ControllerButton::DPadDown => {
    //             self.selected_index += 1;
    //             if self.selected_index >= max {
    //                 self.selected_index = 0;
    //             }

    //             changed = true;
    //         }

    //         ControllerButton::DPadUp => {
    //             if self.selected_index == 0 {
    //                 self.selected_index = max;
    //             } else if self.selected_index >= max { // original value is 99
    //                 self.selected_index = 0;
    //             } else {
    //                 self.selected_index -= 1;
    //             }

    //             changed = true;
    //         }

    //         ControllerButton::South => {
    //             match (self.selected_index, self.is_fail_menu) {
    //                 // continue
    //                 (0, false) => self.unpause(game),
    //                 // retry
    //                 (1, false) | (0, true) => self.retry(game).await,
    //                 // close
    //                 (2, false) | (1, true) => self.exit(game).await,
    //                 _ => {}
    //             }
    //         }

    //         _ => {}
    //     }

    //     trace!("changed:{}, index: {}", changed, self.selected_index);

    //     if changed {
    //         let mut continue_index = 0;
    //         if self.is_fail_menu { continue_index = -1 }
    //         self.continue_button.set_selected(self.selected_index == continue_index);
    //         self.retry_button.set_selected(self.selected_index == continue_index + 1);
    //         self.exit_button.set_selected(self.selected_index == continue_index + 2);
    //     }

    //     true
    // }

}
