use crate::prelude::*;

#[async_trait]
pub trait AsyncMenu:Send+Sync {
    fn get_name(&self) -> &'static str { "none" }

    fn view(&self) -> IcedElement;
    
    async fn handle_message(&mut self, message: Message);
    async fn update(&mut self) -> Vec<MenuAction> { Vec::new() }
    async fn on_change(&mut self, _into:bool) {}// when the menu is "loaded"(into) or "unloaded"(!into)
}
