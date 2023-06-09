// use piston::Event;
// use piston::TextEvent;
// use piston::FocusEvent;
// use piston::ButtonEvent;
// use piston::ResizeEvent;
// use piston::input::Button;
// use piston::MouseScrollEvent;
// use piston::MouseCursorEvent;
// use piston::input::ButtonState;

use crate::prelude::*;
pub struct InputManager {
    pub mouse_pos: Vector2,
    pub scroll_delta: f32,
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
    pub controller_axis: HashMap<u32, HashMap<u8, (bool, f32)>>,

    /// currently pressed keys
    keys: HashSet<Key>,
    /// keys that were pressed but waiting to be registered
    keys_down: HashSet<(Key, Instant)>,
    /// keys that were released but waiting to be registered
    keys_up: HashSet<(Key, Instant)>,
    
    text_cache: String,
    window_change_focus: Option<bool>,
    register_times: Vec<f32>,

    /// do we try to protect against double taps? if so, whats the duration we should check for?
    pub double_tap_protection: Option<f32>,
    
    /// last key pressed, time it was pressed, was it a double tap? (need to know if it was a double tap for release check)
    last_key_press: HashMap<Key, (Instant, bool)>,



    pub raw_input: bool,
}
impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            mouse_pos: Vector2::ZERO,
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

            double_tap_protection: None,
            last_key_press: HashMap::new(),

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

    pub fn set_double_tap_protection(&mut self, protection: Option<f32>) {
        self.double_tap_protection = protection;
    }

    pub fn handle_events(&mut self, e:Window2GameEvent) {

        match e {
            // window events
            Window2GameEvent::GotFocus => self.window_change_focus = Some(true),
            Window2GameEvent::LostFocus => self.window_change_focus = Some(false),

            // GameWindowEvent::Minimized => {},
            // GameWindowEvent::Closed => {}
            // GameWindowEvent::FileHover(_) => {},
            // GameWindowEvent::FileDrop(_) => {},

            // keyboard input
            Window2GameEvent::KeyPress(key) if !self.keys.contains(&key) => {
                
                let mut ok_to_continue = true;

                if let Some(check) = self.double_tap_protection {
                    if let Some((press_time, is_double_tap)) = self.last_key_press.get_mut(&key) {
                        let since = press_time.as_millis();
                        if since <= check {
                            warn!("stopped a doubletap of duration {since:.4}ms");
                            ok_to_continue = false;
                            *is_double_tap = false;
                        }
                    }
                }

                if ok_to_continue {
                    self.keys.insert(key);
                    self.keys_down.insert((key, Instant::now()));
                    self.last_key_press.insert(key, (Instant::now(), false));
                }
            }
            Window2GameEvent::KeyRelease(key) => {
                let mut ok_to_continue = true;

                if self.double_tap_protection.is_some() {
                    if let Some((_, is_double_tap)) = self.last_key_press.get(&key) {
                        if *is_double_tap {
                            ok_to_continue = false;
                        }
                    }
                }
                
                if ok_to_continue {
                    self.keys.remove(&key);
                    self.keys_up.insert((key, Instant::now()));
                    // self.last_key_press.remove(&key);
                } else {
                    self.last_key_press.remove(&key);
                }
            }
            Window2GameEvent::Text(text) => {
                let mods = self.get_key_mods();
                if !mods.alt && !mods.ctrl {
                    self.text_cache += &text
                }
            }

            // mouse input
            Window2GameEvent::MousePress(mb) => {
                self.mouse_buttons.insert(mb);
                self.mouse_down.insert((mb, Instant::now()));
            }
            Window2GameEvent::MouseRelease(mb) => {
                self.mouse_buttons.remove(&mb);
                self.mouse_up.insert((mb, Instant::now()));
            }
            Window2GameEvent::MouseMove(mouse_pos) => {
                if mouse_pos == self.mouse_pos { return }
                self.mouse_moved = true;
                self.mouse_pos = mouse_pos;
            }
            Window2GameEvent::MouseScroll(delta) => self.scroll_delta += delta,

            _ => {}
        }

    }

    // pub fn handle_controller_events(&mut self, e:Event, controller_name: String) {
    //     use input::ControllerAxisEvent;

    //     if let Some(axis) = e.controller_axis_args() {
    //         // debug!("got controller axis: {:?}", axis);
    //         let id = axis.axis;
    //         let value = axis.position;
    //         let controller_id = axis.id;
    //         self.verify_controller_index_exists(controller_id, controller_name);

    //         let map = self.controller_axis.get_mut(&controller_id).unwrap();
    //         if ![Some(&(true, value)), Some(&(false, value))].contains(&map.get(&id)) {
    //             map.insert(id, (true, value));
    //         }
    //     } else if let Some(button) = e.button_args() {
    //         match (button.button, button.state) {
    //             (Button::Controller(cb), ButtonState::Press) => {
    //                 // debug!("press: c: {}, b: {}", cb.id, cb.button);
    //                 self.verify_controller_index_exists(cb.id, controller_name);
    //                 self.controller_buttons.get_mut(&cb.id).unwrap().insert(cb.button);
    //                 self.controller_down.get_mut(&cb.id).unwrap().insert(cb.button);
    //             }
    //             (Button::Controller(cb), ButtonState::Release) => {
    //                 // debug!("release: c: {}, b: {}", cb.id, cb.button);
    //                 self.controller_buttons.get_mut(&cb.id).unwrap().remove(&cb.button);
    //                 self.controller_up.get_mut(&cb.id).unwrap().insert(cb.button);
    //             }
    //             _ => {}
    //         }
    //     }

    // }

    /// is the key currently down (not up)
    pub fn key_down(&self, k:Key) -> bool {self.keys.contains(&k)}
    pub fn get_key_mods(&self) -> KeyModifiers {
        KeyModifiers {
            ctrl: self.key_down(Key::LControl) || self.key_down(Key::RControl),
            alt: self.key_down(Key::LAlt) || self.key_down(Key::RAlt),
            shift: self.key_down(Key::LShift) || self.key_down(Key::RShift),
        }
    }


    /// get all keys that were pressed, and clear the pressed list. (will be true when first checked and pressed, false after first check or when key is up)
    pub fn get_keys_down(&mut self) -> Vec<Key> {
        let mut down = Vec::new();
        for (i, time) in &self.keys_down { down.push(*i); self.register_times.push(time.as_millis()); }
        self.keys_down.clear();

        down
    }
    pub fn get_keys_up(&mut self) -> Vec<Key> {
        let mut up = Vec::new();
        for (i, time) in &self.keys_up { up.push(*i); self.register_times.push(time.as_millis()); }
        self.keys_up.clear();

        up
    }


    /// get all pressed mouse buttons, and reset the pressed array
    pub fn get_mouse_down(&mut self) -> Vec<MouseButton> {
        let mut down = Vec::new();
        for (i, time) in &self.mouse_down { down.push(*i); self.register_times.push(time.as_millis()); }
        self.mouse_down.clear();
        down
    }
    pub fn get_mouse_up(&mut self) -> Vec<MouseButton> {
        let mut up = Vec::new();
        for (i, time) in &self.mouse_up { up.push(*i); self.register_times.push(time.as_millis()); }
        self.mouse_up.clear();
        up
    }

    /// get whether the mouse was moved or not
    pub fn get_mouse_moved(&mut self) -> bool {
        std::mem::take(&mut self.mouse_moved)
    }
    /// get how much the mouse wheel as scrolled (vertically) since the last check
    pub fn get_scroll_delta(&mut self) -> f32 {
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
    pub fn get_controller_axis(&mut self) -> Vec<(Box<dyn Controller>, HashMap<u8, (bool, f32)>)> {
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
