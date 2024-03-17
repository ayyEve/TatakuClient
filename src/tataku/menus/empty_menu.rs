use crate::prelude::*;

pub struct EmptyMenu;
impl EmptyMenu {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl AsyncMenu for EmptyMenu {
    fn view(&self, _values: &mut ValueCollection) -> IcedElement { EmptyElement.into_element() }
    async fn handle_message(&mut self, _message: Message, _values: &mut ValueCollection) {}
}