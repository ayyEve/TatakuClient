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

    fn view(&self, _values: &mut dyn Reflect) -> IcedElement {
        EmptyElement.into_element()
    }
    
    async fn handle_message(&mut self, _message: Message, _values: &mut dyn Reflect) {

    }
    async fn update(&mut self, _values: &mut dyn Reflect) -> Vec<TatakuAction> { 
        Vec::new() 
    }
}
