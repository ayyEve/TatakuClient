use crate::*;
use common::replays::KeyPress;


/// TODO: is it worth moving everything into just a vec instead of using a hashmap?
#[derive(Default, Clone)]
pub struct KeyCounter {
    pub keys: HashMap<KeyPress, KeyInfo>,
    pub key_order: Vec<KeyPress>,
}
impl KeyCounter {
    pub fn new(key_defs: &[(KeyPress, &str)]) -> Self {
        let mut key_order = Vec::new();
        let mut keys = HashMap::new();

        for &(key, label) in key_defs {
            key_order.push(key);
            keys.insert(key, KeyInfo::new(label.to_owned()));
        }

        Self {
            keys,
            key_order
        }
    }

    pub fn key_down(&mut self, key: KeyPress) {
        let Some(info) = self.keys.get_mut(&key) 
        else { return };
        
        info.count += 1;
        info.held = true;
    }
    pub fn key_up(&mut self, key: KeyPress) {
        let Some(info) = self.keys.get_mut(&key) 
        else { return };

        info.held = false;
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
