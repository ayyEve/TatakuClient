impl CommonGameplaySettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();

// key_offset_up
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.key_offset_up, "Increase Offset", font.clone());
i.set_tag(&(prefix.clone() + "key_offset_up"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// key_offset_down
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.key_offset_down, "Decrease Offset", font.clone());
i.set_tag(&(prefix.clone() + "key_offset_down"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// map_restart_key
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.map_restart_key, "Restart Map Key", font.clone());
i.set_tag(&(prefix.clone() + "map_restart_key"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// map_restart_delay
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Restart Map Hold Time", self.map_restart_delay as f64, Some(0.0..1000.0), None, font.clone());
i.set_tag(&(prefix.clone() + "map_restart_delay"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// hit_indicator_draw_duration
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Hit Indicator Draw Time", self.hit_indicator_draw_duration as f64, Some(100.0..1000.0), None, font.clone());
i.set_tag(&(prefix.clone() + "hit_indicator_draw_duration"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// use_indicator_draw_duration_for_animations
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Use Draw Time for Animations", self.use_indicator_draw_duration_for_animations, font.clone());
i.set_tag(&(prefix.clone() + "use_indicator_draw_duration_for_animations"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// key_offset_up

                if let Some(val) = list.get_tagged(prefix.clone() + "key_offset_up").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for key_offset_up"));
                    self.key_offset_up = val.clone(); 
                }

// key_offset_down

                if let Some(val) = list.get_tagged(prefix.clone() + "key_offset_down").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for key_offset_down"));
                    self.key_offset_down = val.clone(); 
                }

// map_restart_key

                if let Some(val) = list.get_tagged(prefix.clone() + "map_restart_key").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for map_restart_key"));
                    self.map_restart_key = val.clone(); 
                }

// map_restart_delay

                if let Some(val) = list.get_tagged(prefix.clone() + "map_restart_delay").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for map_restart_delay"));
                    
                    self.map_restart_delay = (*val) as f32; 
                }

// hit_indicator_draw_duration

                if let Some(val) = list.get_tagged(prefix.clone() + "hit_indicator_draw_duration").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for hit_indicator_draw_duration"));
                    
                    self.hit_indicator_draw_duration = (*val) as f32; 
                }

// use_indicator_draw_duration_for_animations

                if let Some(val) = list.get_tagged(prefix.clone() + "use_indicator_draw_duration_for_animations").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for use_indicator_draw_duration_for_animations"));
                    
                    self.use_indicator_draw_duration_for_animations = val.clone(); 
                }
}
}