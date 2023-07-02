impl IntegrationSettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();

// discord
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Discord Integration", self.discord, font.clone());
i.set_tag(&(prefix.clone() + "discord"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// lastfm
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "LastFM Integration", self.lastfm, font.clone());
i.set_tag(&(prefix.clone() + "lastfm"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// media_controls
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "OS Media Controls", self.media_controls, font.clone());
i.set_tag(&(prefix.clone() + "media_controls"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// discord

                if let Some(val) = list.get_tagged(prefix.clone() + "discord").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for discord"));
                    
                    self.discord = val.clone(); 
                }

// lastfm

                if let Some(val) = list.get_tagged(prefix.clone() + "lastfm").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for lastfm"));
                    
                    self.lastfm = val.clone(); 
                }

// media_controls

                if let Some(val) = list.get_tagged(prefix.clone() + "media_controls").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for media_controls"));
                    
                    self.media_controls = val.clone(); 
                }
}
}