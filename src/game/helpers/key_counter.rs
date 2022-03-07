use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct KeyCounter {
    pos: Vector2,
    keys: HashMap<KeyPress, KeyInfo>,
    key_order: Vec<KeyPress>,

    background_image: Option<Image>,
    button_image: Option<Image>
}
impl KeyCounter {
    pub fn new(key_defs:Vec<(KeyPress, String)>, pos:Vector2) -> Self {
        let mut key_order = Vec::new();
        let mut keys = HashMap::new();

        for (key, label) in key_defs {
            key_order.push(key);
            keys.insert(key, KeyInfo::new(label));
        }

        let mut background_image = SKIN_MANAGER.write().get_texture("inputoverlay-background", true);
        if let Some(image) = &mut background_image {
            image.current_rotation = 90f64.to_radians();
            image.origin = Vector2::new(image.size().x, 0.0);
            // image.current_pos = pos - Vector2::new(image.size().x, 0.0);
            image.depth = -100.0;
        }

        let mut button_image= SKIN_MANAGER.write().get_texture("inputoverlay-key", true);
        if let Some(image) = &mut button_image {
            image.depth = -100.1;
        }

        Self {
            keys,
            key_order,
            pos,
            background_image,
            button_image,
        }
    }

    pub fn key_down(&mut self, key: KeyPress) {
        if self.keys.contains_key(&key) {
            let info = self.keys.get_mut(&key).unwrap();
            info.count += 1;
            info.held = true;
        }
    }
    pub fn key_up(&mut self, key: KeyPress) {
        if self.keys.contains_key(&key) {
            let info = self.keys.get_mut(&key).unwrap();
            info.held = false;
        }
    }

    pub fn reset(&mut self) {
        for i in self.keys.values_mut() {
            i.count = 0;
            i.held = false;
        }
    }


    pub fn draw(&mut self, args: piston::RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let font = get_font("main");
        let window_size:Vector2 = args.window_size.into();

        // let pad = if let Some((btn, bg)) = self.button_image.as_ref().zip(self.background_image.as_ref()) {
        //     let btn_size = btn.size();
        //     let btn_count = self.key_order.len() as f64;
        //     Vector2::new(
        //         btn_size.x,
        //         (bg.size().y - btn_size.y * btn_count) / btn_count
        //     )
        // } else {
        //     BOX_SIZE
        // };
        let pad = BOX_SIZE;
        
        let base_pos = Vector2::new(window_size.x, window_size.y / 2.0) - (self.pos + pad);

        // if let Some(bg) = &self.background_image {
        //     let mut bg = bg.clone();
        //     bg.current_pos = base_pos + Vector2::new(pad.x, pad.y * self.key_order.len() as f64);
        //     list.push(Box::new(bg));
        // }

        //TODO: center properly somehow
        for i in 0..self.key_order.len() {
            let info = &self.keys[&self.key_order[i]];
            let pos = base_pos + Vector2::new(0.0, pad.y * i as f64);

            if let Some(btn) = &self.button_image {
                let mut btn = btn.clone();
                btn.current_pos = pos + pad / 2.0;
                if info.held {
                    btn.current_scale = Vector2::new(1.1, 1.1);
                }
                
                list.push(Box::new(btn));
            } else {
                // draw bg box
                list.push(Box::new(Rectangle::new(
                    if info.held {
                        Color::new(0.8, 0.0, 0.8, 0.8)
                    } else {
                        Color::new(0.0, 0.0, 0.0, 0.8)
                    },
                    -100.0,
                    pos,
                    BOX_SIZE,
                    Some(Border::new(Color::BLACK, 2.0))
                )));
            }

            // draw key
            let mut text = Text::new(
                Color::WHITE,
                -100.1,
                pos,
                20,
                if info.count == 0 {info.label.clone()} else {format!("{}", info.count)},
                font.clone()
            );
            text.center_text(Rectangle::bounds_only(pos, BOX_SIZE));
            list.push(Box::new(text));
        }
    }
}



struct KeyInfo {
    label: String,
    held: bool,
    count: u16,
}
impl KeyInfo {
    fn new(label: String) -> Self {
        Self {
            label,
            held: false,
            count: 0
        }
    }
}