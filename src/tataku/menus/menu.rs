use crate::prelude::*;

#[async_trait]
pub trait AsyncMenu:Send+Sync {
    fn get_name(&self) -> &'static str { "none" }
    fn get_custom_name(&self) -> Option<&String> { None }

    fn view(&self, values: &mut ShuntingYardValues) -> IcedElement;
    
    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues);
    async fn update(&mut self, _values: &mut ShuntingYardValues) -> Vec<MenuAction> { Vec::new() }
    async fn on_change(&mut self, _into:bool) {}// when the menu is "loaded"(into) or "unloaded"(!into)
}
