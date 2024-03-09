use crate::prelude::*;

pub struct SpectatorMenu {

}

impl SpectatorMenu {
    pub fn new() -> Self {
        Self {
            
        }
    }
}

#[async_trait]
impl AsyncMenu for SpectatorMenu {
    fn get_name(&self) -> &'static str { "spectator_menu" }

    fn view(&self, _values: &ShuntingYardValues) -> IcedElement {
        use iced_elements::*;
    
        EmptyElement.into_element()
    }
    
    async fn handle_message(&mut self, _message: Message) {

    }
    async fn update(&mut self) -> Vec<MenuAction> { 
        Vec::new() 
    }
}
