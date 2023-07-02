impl LoggingSettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();

// extra_online_logging
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Extra Online Logging", self.extra_online_logging, font.clone());
i.set_tag(&(prefix.clone() + "extra_online_logging"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// extra_online_logging

                if let Some(val) = list.get_tagged(prefix.clone() + "extra_online_logging").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for extra_online_logging"));
                    
                    self.extra_online_logging = val.clone(); 
                }
}
}