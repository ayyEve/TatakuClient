impl ManiaSettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {
}
}