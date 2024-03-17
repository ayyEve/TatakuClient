use crate::prelude::*;

pub struct BeatmapDialog {
    actions: ActionQueue,
    num: usize,
    target_map: Md5Hash,
    should_close: bool,
}
impl BeatmapDialog {
    pub fn new(target_map: Md5Hash) -> Self {
        Self {
            actions: ActionQueue::new(),
            num: 0,
            
            target_map,
            should_close: false
        }
    }
}

#[async_trait]
impl Dialog for BeatmapDialog {
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }

    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.should_close = true; }


    fn view(&self) -> IcedElement {
        use iced_elements::*;

        col!(
            // delete map
            Button::new(Text::new("Delete Map")).on_press(Message::new_dialog(self, "delete", MessageType::Click)),

            // copy_hash
            Button::new(Text::new("Copy Hash")).on_press(Message::new_dialog(self, "copy_hash", MessageType::Click));

            height = Fill
        )

    }

    async fn handle_message(&mut self, message: Message, _values: &mut ValueCollection) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &*tag {
            "delete" => {
                self.actions.push(BeatmapAction::Delete(self.target_map));
                self.should_close = true;
            }

            "copy_hash" => {
                trace!("copy hash map {}", self.target_map);
                match GameWindow::set_clipboard(self.target_map.to_string()) {
                    Ok(_) => NotificationManager::add_text_notification("Hash copied to clipboard!", 3000.0, Color::LIGHT_BLUE).await,
                    Err(e) => NotificationManager::add_error_notification("Failed to copy hash to clipboard", e).await,
                }

                self.should_close = true;
            }

            _ => {}
        }
    }
    
    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> { self.actions.take() }
}
