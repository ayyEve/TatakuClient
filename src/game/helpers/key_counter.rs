use crate::prelude::*;

const BOX_SIZE:Vector2 = Vector2::new(40.0, 40.0);

pub struct KeyCounter {
    pos: Vector2,
    keys: HashMap<KeyPress, KeyInfo>,
    key_order: Vec<KeyPress>
}
impl KeyCounter {
    pub fn new(key_defs:Vec<(KeyPress, String)>, pos:Vector2) -> Self {
        let mut key_order = Vec::new();
        let mut keys = HashMap::new();

        for (key, label) in key_defs {
            key_order.push(key);
            keys.insert(key, KeyInfo::new(label));
        }

        Self {
            keys,
            key_order,
            pos
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


    pub fn draw(&mut self, args: piston::RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let font = get_font("main");
        let window_size:Vector2 = args.window_size.into();

        //TODO: center properly somehow
        for i in 0..self.key_order.len() {
            let info = &self.keys[&self.key_order[i]];
            let pos = (Vector2::new(window_size.x - BOX_SIZE.x, window_size.y / 2.0 - BOX_SIZE.y) - self.pos) + Vector2::new(0.0, BOX_SIZE.y * i as f64);

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