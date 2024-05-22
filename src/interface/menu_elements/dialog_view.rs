use crate::prelude::*;


/// widget for drawing all dialogs
#[derive(Default)]
pub struct DialogManager {
    dialog_counter: usize,
    pub dialogs: Vec<Box<dyn Dialog>>,
}
impl DialogManager {
    pub fn new() -> Self { Self::default() }

    pub fn add_dialog(&mut self, mut dialog: Box<dyn Dialog>) {
        dialog.set_num(self.dialog_counter);
        self.dialog_counter += 1;
        self.dialogs.push(dialog);
    }

    pub async fn close_latest(&mut self) -> bool {
        let Some(last) = self.dialogs.last_mut() else { return false };
        last.force_close().await;
        true
    }

    pub async fn handle_message(&mut self, message: Message, values: &mut ValueCollection) {
        for d in self.dialogs.iter_mut() {
            if message.owner.check_dialog(d) {
                d.handle_message(message, values).await;
                return
            }
        }
    }

    pub async fn force_close_all(&mut self) {
        for i in self.dialogs.iter_mut() {
            i.force_close().await
        }
    }


    pub async fn update(&mut self, values: &mut ValueCollection) -> Vec<TatakuAction> {
        let mut list = Vec::new();
        for i in self.dialogs.iter_mut() {
            list.extend(i.update(values).await);
        }
        self.dialogs.retain(|d|!d.should_close());
        
        list
    }

    pub fn view(&self, values: &mut ValueCollection) -> IcedElement {
        use iced_elements::*;

        let dialogs = self.dialogs.iter().map(|d|DraggableDialogElement::new(d, values).into_element()).collect();
        col!(
            dialogs,
        )
    }
}
