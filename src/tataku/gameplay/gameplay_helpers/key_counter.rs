use crate::prelude::*;

#[derive(Default, Clone)]
pub struct KeyCounter {
    pub keys: HashMap<KeyPress, KeyInfo>,
    pub key_order: Vec<KeyPress>,
}
impl KeyCounter {
    pub fn new(key_defs:Vec<(KeyPress, String)>) -> Self {
        let mut key_order = Vec::new();
        let mut keys = HashMap::new();

        for (key, label) in key_defs {
            key_order.push(key);
            keys.insert(key, KeyInfo::new(label));
        }


        Self {
            keys,
            key_order
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
}


#[derive(Clone)]
pub struct KeyInfo {
    pub label: String,
    pub held: bool,
    pub count: u16,
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