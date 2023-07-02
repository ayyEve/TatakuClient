impl StandardSettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();

// left_key
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.left_key, "Osu Key 1", font.clone());
i.set_tag(&(prefix.clone() + "left_key"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// right_key
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.right_key, "Osu Key 2", font.clone());
i.set_tag(&(prefix.clone() + "right_key"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// ignore_mouse_buttons
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Ignore Mouse Buttons", self.ignore_mouse_buttons, font.clone());
i.set_tag(&(prefix.clone() + "ignore_mouse_buttons"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// manual_input_with_relax
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Allow manual input with Relax", self.manual_input_with_relax, font.clone());
i.set_tag(&(prefix.clone() + "manual_input_with_relax"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// draw_follow_points
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Follow Points", self.draw_follow_points, font.clone());
i.set_tag(&(prefix.clone() + "draw_follow_points"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// show_300s
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Show x300s", self.show_300s, font.clone());
i.set_tag(&(prefix.clone() + "show_300s"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// hit_ripples
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Hit Ripples", self.hit_ripples, font.clone());
i.set_tag(&(prefix.clone() + "hit_ripples"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// slider_tick_ripples
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Slider Tick Ripples", self.slider_tick_ripples, font.clone());
i.set_tag(&(prefix.clone() + "slider_tick_ripples"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// ripple_hitcircles
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Ripple HitCircles", self.ripple_hitcircles, font.clone());
i.set_tag(&(prefix.clone() + "ripple_hitcircles"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// ripple_scale
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Ripple Scale", self.ripple_scale as f64, Some(0.1..5.0), None, font.clone());
i.set_tag(&(prefix.clone() + "ripple_scale"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// slider_tick_ripples_above
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Slider Tick Ripples Above", self.slider_tick_ripples_above, font.clone());
i.set_tag(&(prefix.clone() + "slider_tick_ripples_above"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// approach_combo_color
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Combo Color Approach Circles", self.approach_combo_color, font.clone());
i.set_tag(&(prefix.clone() + "approach_combo_color"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// use_beatmap_combo_colors
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Beatmap Combo Colors", self.use_beatmap_combo_colors, font.clone());
i.set_tag(&(prefix.clone() + "use_beatmap_combo_colors"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// use_skin_judgments
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Use Skin Judgments", self.use_skin_judgments, font.clone());
i.set_tag(&(prefix.clone() + "use_skin_judgments"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// slider_body_alpha
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Slider Body Alpha", self.slider_body_alpha as f64, Some(0.00001..1.0), None, font.clone());
i.set_tag(&(prefix.clone() + "slider_body_alpha"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// slider_border_alpha
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Slider Border Alpha", self.slider_border_alpha as f64, Some(0.0..1.0), None, font.clone());
i.set_tag(&(prefix.clone() + "slider_border_alpha"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// playfield_alpha
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Playfield Alpha", self.playfield_alpha as f64, Some(0.0..1.0), None, font.clone());
i.set_tag(&(prefix.clone() + "playfield_alpha"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// left_key

                if let Some(val) = list.get_tagged(prefix.clone() + "left_key").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for left_key"));
                    self.left_key = val.clone(); 
                }

// right_key

                if let Some(val) = list.get_tagged(prefix.clone() + "right_key").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for right_key"));
                    self.right_key = val.clone(); 
                }

// ignore_mouse_buttons

                if let Some(val) = list.get_tagged(prefix.clone() + "ignore_mouse_buttons").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for ignore_mouse_buttons"));
                    
                    self.ignore_mouse_buttons = val.clone(); 
                }

// manual_input_with_relax

                if let Some(val) = list.get_tagged(prefix.clone() + "manual_input_with_relax").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for manual_input_with_relax"));
                    
                    self.manual_input_with_relax = val.clone(); 
                }

// draw_follow_points

                if let Some(val) = list.get_tagged(prefix.clone() + "draw_follow_points").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for draw_follow_points"));
                    
                    self.draw_follow_points = val.clone(); 
                }

// show_300s

                if let Some(val) = list.get_tagged(prefix.clone() + "show_300s").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for show_300s"));
                    
                    self.show_300s = val.clone(); 
                }

// hit_ripples

                if let Some(val) = list.get_tagged(prefix.clone() + "hit_ripples").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for hit_ripples"));
                    
                    self.hit_ripples = val.clone(); 
                }

// slider_tick_ripples

                if let Some(val) = list.get_tagged(prefix.clone() + "slider_tick_ripples").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for slider_tick_ripples"));
                    
                    self.slider_tick_ripples = val.clone(); 
                }

// ripple_hitcircles

                if let Some(val) = list.get_tagged(prefix.clone() + "ripple_hitcircles").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for ripple_hitcircles"));
                    
                    self.ripple_hitcircles = val.clone(); 
                }

// ripple_scale

                if let Some(val) = list.get_tagged(prefix.clone() + "ripple_scale").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for ripple_scale"));
                    
                    self.ripple_scale = (*val) as f32; 
                }

// slider_tick_ripples_above

                if let Some(val) = list.get_tagged(prefix.clone() + "slider_tick_ripples_above").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for slider_tick_ripples_above"));
                    
                    self.slider_tick_ripples_above = val.clone(); 
                }

// approach_combo_color

                if let Some(val) = list.get_tagged(prefix.clone() + "approach_combo_color").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for approach_combo_color"));
                    
                    self.approach_combo_color = val.clone(); 
                }

// use_beatmap_combo_colors

                if let Some(val) = list.get_tagged(prefix.clone() + "use_beatmap_combo_colors").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for use_beatmap_combo_colors"));
                    
                    self.use_beatmap_combo_colors = val.clone(); 
                }

// use_skin_judgments

                if let Some(val) = list.get_tagged(prefix.clone() + "use_skin_judgments").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for use_skin_judgments"));
                    
                    self.use_skin_judgments = val.clone(); 
                }

// slider_body_alpha

                if let Some(val) = list.get_tagged(prefix.clone() + "slider_body_alpha").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for slider_body_alpha"));
                    
                    self.slider_body_alpha = (*val) as f32; 
                }

// slider_border_alpha

                if let Some(val) = list.get_tagged(prefix.clone() + "slider_border_alpha").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for slider_border_alpha"));
                    
                    self.slider_border_alpha = (*val) as f32; 
                }

// playfield_alpha

                if let Some(val) = list.get_tagged(prefix.clone() + "playfield_alpha").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for playfield_alpha"));
                    
                    self.playfield_alpha = (*val) as f32; 
                }
}
}