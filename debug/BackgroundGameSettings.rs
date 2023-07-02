impl BackgroundGameSettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();

// main_menu_enabled
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Main Menu Background Gameplay", self.main_menu_enabled, font.clone());
i.set_tag(&(prefix.clone() + "main_menu_enabled"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// beatmap_select_enabled
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Map Select Background Gameplay", self.beatmap_select_enabled, font.clone());
i.set_tag(&(prefix.clone() + "beatmap_select_enabled"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// settings_menu_enabled
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Settings Background Gameplay", self.settings_menu_enabled, font.clone());
i.set_tag(&(prefix.clone() + "settings_menu_enabled"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// main_menu_enabled

                if let Some(val) = list.get_tagged(prefix.clone() + "main_menu_enabled").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for main_menu_enabled"));
                    
                    self.main_menu_enabled = val.clone(); 
                }

// beatmap_select_enabled

                if let Some(val) = list.get_tagged(prefix.clone() + "beatmap_select_enabled").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for beatmap_select_enabled"));
                    
                    self.beatmap_select_enabled = val.clone(); 
                }

// settings_menu_enabled

                if let Some(val) = list.get_tagged(prefix.clone() + "settings_menu_enabled").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for settings_menu_enabled"));
                    
                    self.settings_menu_enabled = val.clone(); 
                }
}
}