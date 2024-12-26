use crate::prelude::*;

/// handles all the ui things
pub struct UiApplication {
    /// what menu is currently being drawn?
    pub menu: Box<dyn AsyncMenu>,
    /// what dialogs are visible?
    pub dialog_manager: DialogManager,
}
impl UiApplication {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            menu: Box::new(EmptyMenu::new()),
            dialog_manager: DialogManager::new(),
        }
    }

    pub async fn update(&mut self, values: &mut dyn Reflect) -> Vec<TatakuAction> {
        self.menu.update(values).await
    }
    pub async fn handle_event(&mut self, event: TatakuEventType, param: Option<TatakuValue>, values: &mut dyn Reflect) {
        self.menu.handle_event(event, param, values).await;
    }

    pub async fn handle_message(&mut self, message: Message, values: &mut dyn Reflect) {
        if message.owner.is_menu() {
            self.menu.handle_message(message, values).await;
        } else {
            self.dialog_manager.handle_message(message, values).await;
        }
    }


    pub fn view(&self, values: &mut dyn Reflect) -> IcedElement {
        use crate::prelude::iced_elements::*;
        let content:IcedElement = self.menu.view(values);
        let dialogs = self.dialog_manager.view(values);

        Container::new(col!(
            content,
            dialogs;
            width = Fill,
            height = Fill
        ))
            // .width(Fill)
            // .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into_element()
    }

}