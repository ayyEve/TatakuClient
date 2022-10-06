use piston::Event;
use piston::ResizeEvent;
use piston::TextEvent;
use piston::FocusEvent;
use piston::ButtonEvent;
use piston::input::Button;
use piston::MouseScrollEvent;
use piston::MouseCursorEvent;
use piston::input::ButtonState;
use piston::ControllerAxisEvent;

use crate::prelude::*;

// #[derive(Clone, Hash, PartialEq, Eq)]
// pub struct ControllerMeta {
//     pub id: u32,
//     pub name: Arc<String>
// }
// impl ControllerMeta {
//     fn new(id:u32, name:Arc<String>) -> Self {
//         Self {
//             id,
//             name
//         }
//     }
// }


pub struct InputManager {
    pub mouse_pos: Vector2,
    pub scroll_delta: f64,
    pub mouse_moved: bool,

    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_down: HashSet<(MouseButton, Instant)>,
    pub mouse_up: HashSet<(MouseButton, Instant)>,

    /// controller names
    pub controller_names: HashMap<u32, Arc<String>>,
    /// index is controller id
    pub controller_buttons: HashMap<u32, HashSet<u8>>,
    /// index is controller id
    pub controller_down: HashMap<u32, HashSet<u8>>,
    /// index is controller id
    pub controller_up: HashMap<u32, HashSet<u8>>,
    /// index is controller id
    /// value index is axis id, value value is (changed, value)
    pub controller_axis: HashMap<u32, HashMap<u8, (bool, f64)>>,

    /// currently pressed keys
    keys: HashSet<Key>,
    /// keys that were pressed but waiting to be registered
    keys_down: HashSet<(Key, Instant)>,
    /// keys that were released but waiting to be registered
    keys_up: HashSet<(Key, Instant)>,
    
    text_cache: String,
    window_change_focus: Option<bool>,
    register_times: Vec<f32>,

    pub raw_input: bool
}
impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            mouse_pos: Vector2::zero(),
            scroll_delta: 0.0,
            mouse_moved: false,
            register_times: Vec::new(),

            mouse_buttons: HashSet::new(),
            mouse_down: HashSet::new(),
            mouse_up: HashSet::new(),

            keys: HashSet::new(),
            keys_down: HashSet::new(),
            keys_up:  HashSet::new(),

            
            controller_names: HashMap::new(),
            controller_buttons: HashMap::new(),
            controller_down: HashMap::new(),
            controller_up: HashMap::new(),
            controller_axis: HashMap::new(),


            text_cache: String::new(),
            window_change_focus: None,

            raw_input: false
        }
    }

    
    fn verify_controller_index_exists(&mut self, id: u32, name:String) {
        if !self.controller_names.contains_key(&id) {
            // window.joystick_deadzone = 0.01;
            debug!("New controller: {}", name);
            self.controller_names.insert(id, Arc::new(name));
        }

        if !self.controller_buttons.contains_key(&id) {
            self.controller_buttons.insert(id, HashSet::new());
        }

        if !self.controller_down.contains_key(&id) {
            self.controller_down.insert(id, HashSet::new());
        }

        if !self.controller_up.contains_key(&id) {
            self.controller_up.insert(id, HashSet::new());
        }
    
        if !self.controller_axis.contains_key(&id) {
            self.controller_axis.insert(id, HashMap::new());
        }
    }


    pub fn handle_events(&mut self, e:Event) {
        if let Some(button) = e.button_args() {
            match (button.button, button.state) {
                (Button::Keyboard(key), ButtonState::Press) => {
                    self.keys.insert(key);
                    self.keys_down.insert((key, Instant::now()));
                }
                (Button::Keyboard(key), ButtonState::Release) => {
                    self.keys.remove(&key);
                    self.keys_up.insert((key, Instant::now()));
                }
                (Button::Mouse(mb), ButtonState::Press) => {
                    self.mouse_buttons.insert(mb);
                    self.mouse_down.insert((mb, Instant::now()));
                }
                (Button::Mouse(mb), ButtonState::Release) => {
                    self.mouse_buttons.remove(&mb);
                    self.mouse_up.insert((mb, Instant::now()));
                }

                _ => {}
            }
        }

        // if let Some(axis) = e.controller_axis_args() {
        //     // debug!("got controller axis: {:?}", axis);
        //     let id = axis.axis;
        //     let value = axis.position;
        //     let controller_id = axis.id;
        //     self.verify_controller_index_exists(controller_id, window);

        //     let map = self.controller_axis.get_mut(&controller_id).unwrap();
        //     if ![Some(&(true, value)), Some(&(false, value))].contains(&map.get(&id)) {
        //         map.insert(id, (true, value));
        //     }
        // }

        e.mouse_cursor(|[x, y]| {
            let incoming = Vector2::new(x, y);
            if incoming == self.mouse_pos { return }
            self.mouse_moved = true;
            self.mouse_pos = incoming;
        });

        e.mouse_scroll(|d| { self.scroll_delta += d[1]} );
        if let Some(e) = e.text_args() { self.text_cache += &e; }
        if let Some(has_focus) = e.focus_args() { self.window_change_focus = Some(has_focus); }
        if let Some(thing) = e.resize_args() { 
            set_window_size(thing.window_size.into());
        }

        // e.text(|text| debug!("Typed '{}'", text));
    }

    pub fn handle_controller_events(&mut self, e:Event, controller_name: String) {

        if let Some(axis) = e.controller_axis_args() {
            // debug!("got controller axis: {:?}", axis);
            let id = axis.axis;
            let value = axis.position;
            let controller_id = axis.id;
            self.verify_controller_index_exists(controller_id, controller_name);

            let map = self.controller_axis.get_mut(&controller_id).unwrap();
            if ![Some(&(true, value)), Some(&(false, value))].contains(&map.get(&id)) {
                map.insert(id, (true, value));
            }
        } else if let Some(button) = e.button_args() {
            match (button.button, button.state) {
                (Button::Controller(cb), ButtonState::Press) => {
                    // debug!("press: c: {}, b: {}", cb.id, cb.button);
                    self.verify_controller_index_exists(cb.id, controller_name);
                    self.controller_buttons.get_mut(&cb.id).unwrap().insert(cb.button);
                    self.controller_down.get_mut(&cb.id).unwrap().insert(cb.button);
                }
                (Button::Controller(cb), ButtonState::Release) => {
                    // debug!("release: c: {}, b: {}", cb.id, cb.button);
                    self.controller_buttons.get_mut(&cb.id).unwrap().remove(&cb.button);
                    self.controller_up.get_mut(&cb.id).unwrap().insert(cb.button);
                }
                _ => {}
            }
        }

    }

    /// is the key currently down (not up)
    pub fn key_down(&self, k:Key) -> bool {self.keys.contains(&k)}
    pub fn get_key_mods(&self) -> KeyModifiers {
        KeyModifiers {
            ctrl: self.key_down(Key::LCtrl) || self.key_down(Key::RCtrl),
            alt: self.key_down(Key::LAlt) || self.key_down(Key::RAlt),
            shift: self.key_down(Key::LShift) || self.key_down(Key::RShift),
        }
    }


    /// get all keys that were pressed, and clear the pressed list. (will be true when first checked and pressed, false after first check or when key is up)
    pub fn get_keys_down(&mut self) -> Vec<Key> {
        let mut down = Vec::new();
        for (i, time) in &self.keys_down {down.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.keys_down.clear();

        down
    }
    pub fn get_keys_up(&mut self) -> Vec<Key> {
        let mut up = Vec::new();
        for (i, time) in &self.keys_up {up.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.keys_up.clear();

        up
    }


    /// get all pressed mouse buttons, and reset the pressed array
    pub fn get_mouse_down(&mut self) -> Vec<MouseButton> {
        let mut down = Vec::new();
        for (i, time) in &self.mouse_down {down.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.mouse_down.clear();
        down
    }
    pub fn get_mouse_up(&mut self) -> Vec<MouseButton> {
        let mut up = Vec::new();
        for (i, time) in &self.mouse_up {up.push(i.clone()); self.register_times.push(time.elapsed().as_secs_f32()*1000.0)}
        self.mouse_up.clear();
        up
    }

    /// get whether the mouse was moved or not
    pub fn get_mouse_moved(&mut self) -> bool {
        std::mem::take(&mut self.mouse_moved)
    }
    /// get how much the mouse wheel as scrolled (vertically) since the last check
    pub fn get_scroll_delta(&mut self) -> f64 {
        std::mem::take(&mut self.scroll_delta)
    }


    /// get all pressed controller buttons, and reset the pressed array
    /// (controller_id, button_id)
    pub fn get_controller_down(&mut self) -> Vec<(Box<dyn Controller>, u8)> {
        let mut down = Vec::new();
        for (c, buttons) in self.controller_down.iter_mut() {
            let name = self.controller_names.get(c).unwrap();
           
            for b in buttons.iter() {
                let controller = make_controller(*c, name.clone());
                down.push((controller, *b));
            }
            buttons.clear()
        }
        down
    }

    /// get all released controller buttons, and reset the pressed array
    /// (controller_id, button_id)
    pub fn get_controller_up(&mut self) -> Vec<(Box<dyn Controller>, u8)> {
        let mut up = Vec::new();
        for (c, buttons) in self.controller_up.iter_mut() {
            let name = self.controller_names.get(c).unwrap();
            
            for b in buttons.iter() {
                let controller = make_controller(*c, name.clone());
                up.push((controller, *b));
            }
            buttons.clear()
        }
        up
    }

    /// get all controller axes
    /// (controller, [axis_id, (changed, value)])
    pub fn get_controller_axis(&mut self) -> Vec<(Box<dyn Controller>, HashMap<u8, (bool, f64)>)> {
        let mut axis = Vec::new();

        for (c, axis_data) in self.controller_axis.iter_mut() {
            let name = self.controller_names.get(c).unwrap();
            let controller = make_controller(*c, name.clone());
            axis.push((controller, axis_data.clone()));

            // update all the changed to false, since we've now checked them
            for (_, (changed, _)) in axis_data.iter_mut() {
                *changed = false
            }
        }

        axis
    }
    
    /// gets any text typed since the last check
    pub fn get_text(&mut self) -> String {
        std::mem::take(&mut self.text_cache)
    }

    /// get whether the window's focus has changed
    pub fn get_changed_focus(&mut self) -> Option<bool> {
        std::mem::take(&mut self.window_change_focus)
    }

    /// get the input register delay average 
    /// (min,max,avg)
    #[allow(unused)]
    pub fn get_register_delay(&mut self) -> (f32,f32,f32) {
        let mut sum = 0.0;
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for i in self.register_times.iter() {
            sum += i;
            min = min.min(*i);
            max = max.max(*i);
        }
        sum /= self.register_times.len() as f32;
        self.register_times.clear();

        (min,max,sum)
    }
}



#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AxisConfig {
    pub axis_id: u8,
    pub threshhold: f64
}


#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ControllerInputConfig {
    pub button: Option<u8>,
    pub axis: Option<AxisConfig>
}
impl ControllerInputConfig {
    pub fn new(button: Option<u8>, axis: Option<AxisConfig>) -> Self {
        Self {
            button, 
            axis
        }
    }

    pub fn check_button(&self, button: u8) -> bool {
        if let Some(b) = self.button {
            b == button
        } else {
            false
        }
    }
}
