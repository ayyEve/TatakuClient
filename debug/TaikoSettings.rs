impl TaikoSettings {
pub fn get_menu_items(&self, p: Vector2, prefix: String, sender: Arc<SyncSender<()>>) -> Vec<Box<dyn ScrollableItem>> {
let mut list:Vec<Box<dyn ScrollableItem>> = Vec::new();
let font = get_font();

// left_kat
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.left_kat, "Left Kat", font.clone());
i.set_tag(&(prefix.clone() + "left_kat"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// left_don
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.left_don, "Left Don", font.clone());
i.set_tag(&(prefix.clone() + "left_don"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// right_don
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.right_don, "Right Don", font.clone());
i.set_tag(&(prefix.clone() + "right_don"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// right_kat
let mut i = KeyButton::new(p, Vector2::new(600.0, 50.0), self.right_kat, "Right Kat", font.clone());
i.set_tag(&(prefix.clone() + "right_kat"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// ignore_mouse_buttons
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Ignore Mouse Buttons", self.ignore_mouse_buttons, font.clone());
i.set_tag(&(prefix.clone() + "ignore_mouse_buttons"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// sv_multiplier
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "SV Multiplier", self.sv_multiplier as f64, Some(1.0..2.0), None, font.clone());
i.set_tag(&(prefix.clone() + "sv_multiplier"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// note_radius
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Note Radius", self.note_radius as f64, Some(1.0..100.0), None, font.clone());
i.set_tag(&(prefix.clone() + "note_radius"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// big_note_multiplier
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Big Note Scale", self.big_note_multiplier as f64, Some(1.0..5.0), None, font.clone());
i.set_tag(&(prefix.clone() + "big_note_multiplier"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// playfield_x_offset
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Playfield Horizontal Offset", self.playfield_x_offset as f64, Some(0.0..500.0), None, font.clone());
i.set_tag(&(prefix.clone() + "playfield_x_offset"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// playfield_y_offset
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Playfield Vertical Offset", self.playfield_y_offset as f64, Some(0.0..200.0), None, font.clone());
i.set_tag(&(prefix.clone() + "playfield_y_offset"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// hit_area_radius_mult
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Hit Area Radius Scale", self.hit_area_radius_mult as f64, Some(1.0..5.0), None, font.clone());
i.set_tag(&(prefix.clone() + "hit_area_radius_mult"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// playfield_height_padding
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Playfield Vertical Padding", self.playfield_height_padding as f64, Some(0.0..20.0), None, font.clone());
i.set_tag(&(prefix.clone() + "playfield_height_padding"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// don_color
let s:String = self.don_color.into(); let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Don Color", &s, font.clone());
i.set_tag(&(prefix.clone() + "don_color"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// kat_color
let s:String = self.kat_color.into(); let mut i = TextInput::new(p, Vector2::new(600.0, 50.0), "Kat Color", &s, font.clone());
i.set_tag(&(prefix.clone() + "kat_color"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// use_skin_judgments
let mut i = Checkbox::new(p, Vector2::new(600.0, 50.0), "Use Skin Judgments", self.use_skin_judgments, font.clone());
i.set_tag(&(prefix.clone() + "use_skin_judgments"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));

// judgement_indicator_offset
let mut i = Slider::new(p, Vector2::new(600.0, 50.0), "Hit Judgment Y-Offset", self.judgement_indicator_offset as f64, Some(0.0..100.0), None, font.clone());
i.set_tag(&(prefix.clone() + "judgement_indicator_offset"));
let c = sender.clone();
i.on_change = Arc::new(move|_,_|{c.send(()).unwrap()});
list.push(Box::new(i));
list
}
pub fn from_menu(&mut self, prefix: String, list: &ScrollableArea) {

// left_kat

                if let Some(val) = list.get_tagged(prefix.clone() + "left_kat").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for left_kat"));
                    self.left_kat = val.clone(); 
                }

// left_don

                if let Some(val) = list.get_tagged(prefix.clone() + "left_don").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for left_don"));
                    self.left_don = val.clone(); 
                }

// right_don

                if let Some(val) = list.get_tagged(prefix.clone() + "right_don").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for right_don"));
                    self.right_don = val.clone(); 
                }

// right_kat

                if let Some(val) = list.get_tagged(prefix.clone() + "right_kat").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<Key>().expect(&format!("error downcasting for right_kat"));
                    self.right_kat = val.clone(); 
                }

// ignore_mouse_buttons

                if let Some(val) = list.get_tagged(prefix.clone() + "ignore_mouse_buttons").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for ignore_mouse_buttons"));
                    
                    self.ignore_mouse_buttons = val.clone(); 
                }

// sv_multiplier

                if let Some(val) = list.get_tagged(prefix.clone() + "sv_multiplier").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for sv_multiplier"));
                    
                    self.sv_multiplier = (*val) as f32; 
                }

// note_radius

                if let Some(val) = list.get_tagged(prefix.clone() + "note_radius").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for note_radius"));
                    
                    self.note_radius = (*val) as f32; 
                }

// big_note_multiplier

                if let Some(val) = list.get_tagged(prefix.clone() + "big_note_multiplier").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for big_note_multiplier"));
                    
                    self.big_note_multiplier = (*val) as f32; 
                }

// playfield_x_offset

                if let Some(val) = list.get_tagged(prefix.clone() + "playfield_x_offset").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for playfield_x_offset"));
                    
                    self.playfield_x_offset = (*val) as f32; 
                }

// playfield_y_offset

                if let Some(val) = list.get_tagged(prefix.clone() + "playfield_y_offset").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for playfield_y_offset"));
                    
                    self.playfield_y_offset = (*val) as f32; 
                }

// hit_area_radius_mult

                if let Some(val) = list.get_tagged(prefix.clone() + "hit_area_radius_mult").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for hit_area_radius_mult"));
                    
                    self.hit_area_radius_mult = (*val) as f32; 
                }

// playfield_height_padding

                if let Some(val) = list.get_tagged(prefix.clone() + "playfield_height_padding").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for playfield_height_padding"));
                    
                    self.playfield_height_padding = (*val) as f32; 
                }

// don_color

                {
                    let val = list.get_tagged(prefix.clone() + "don_color"); // get item from list
                    let val = val.first().expect("error getting tagged"); // unwrap
                    let val = val.get_value(); // get the value from the item
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for Color (String)"));
                    
                    self.don_color = val.clone().into(); 
                }

// kat_color

                {
                    let val = list.get_tagged(prefix.clone() + "kat_color"); // get item from list
                    let val = val.first().expect("error getting tagged"); // unwrap
                    let val = val.get_value(); // get the value from the item
                    let val = val.downcast_ref::<String>().expect(&format!("error downcasting for Color (String)"));
                    
                    self.kat_color = val.clone().into(); 
                }

// use_skin_judgments

                if let Some(val) = list.get_tagged(prefix.clone() + "use_skin_judgments").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<bool>().expect(&format!("error downcasting for use_skin_judgments"));
                    
                    self.use_skin_judgments = val.clone(); 
                }

// judgement_indicator_offset

                if let Some(val) = list.get_tagged(prefix.clone() + "judgement_indicator_offset").first().map(|i|i.get_value()) {
                    let val = val.downcast_ref::<f64>().expect(&format!("error downcasting for judgement_indicator_offset"));
                    
                    self.judgement_indicator_offset = (*val) as f32; 
                }
}
}