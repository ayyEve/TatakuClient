use crate::prelude::*;

/// handles all the ui things
pub struct UiApplication {
    /// what menu is currently being drawn?
    pub menu: Box<dyn AsyncMenu>,
    /// what dialogs are visible?
    pub dialog_manager: DialogManager,
}
impl UiApplication {
    pub fn new() -> Self {
        Self {
            menu: Box::new(EmptyMenu::new()),
            dialog_manager: DialogManager::new(),
        }
    }

    pub async fn update(&mut self, values: &mut ValueCollection) -> Vec<TatakuAction> {
        self.menu.update(values).await
    }

    pub async fn handle_message(&mut self, message: Message, values: &mut ValueCollection) {
        if message.owner.check_menu(&self.menu) {
            self.menu.handle_message(message, values).await;
        } else {
            self.dialog_manager.handle_message(message, values).await;
        }
    }

    pub async fn handle_make_userpanel(&mut self) {
        let mut user_panel_exists = false;
        let mut chat_exists = false;
        for i in self.dialog_manager.dialogs.iter() {
            if i.name() == "UserPanel" {
                user_panel_exists = true;
            }
            if i.name() == "Chat" {
                chat_exists = true;
            }
            // if both exist, no need to continue looping
            if user_panel_exists && chat_exists { break }
        }

        if !user_panel_exists {
            // close existing chat window
            if chat_exists {
                self.dialog_manager.dialogs.retain(|d|d.name() != "Chat");
            }
            
            self.dialog_manager.add_dialog(Box::new(UserPanel::new()));
        } else {
            self.dialog_manager.dialogs.retain(|d|d.name() != "UserPanel");
        }

        // if let Some(chat) = Chat::new() {
        //     self.add_dialog(Box::new(chat));
        // }
        // trace!("Show user list: {}", self.show_user_list);
    }


    pub fn view(&self, values: &mut ValueCollection) -> IcedElement {
        use crate::prelude::iced_elements::*;
        let content:IcedElement = self.menu.view(values);
        let dialogs = self.dialog_manager.view();

        Container::new(col!(
            content, 
            dialogs;
            width = Fill,
            height = Fill
        ))
            .width(Fill)
            .height(Fill)
            .center_x()
            .center_y()
            .into_element()
    }

}