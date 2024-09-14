use gilrs::PowerInfo;

use crate::prelude::*;

pub struct InputManager {
    pub mouse_pos: Vector2,
    pub scroll_delta: f32,
    pub mouse_moved: bool,

    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_down: HashSet<(MouseButton, Instant)>,
    pub mouse_up: HashSet<(MouseButton, Instant)>,

    /// controller names
    pub controller_info: HashMap<GamepadId, GamepadInfo>,

    /// index is controller id
    pub controller_buttons: HashMap<GamepadId, HashSet<ControllerButton>>,
    /// index is controller id
    pub controller_down: HashMap<GamepadId, HashSet<ControllerButton>>,
    /// index is controller id
    pub controller_up: HashMap<GamepadId, HashSet<ControllerButton>>,
    /// index is controller id
    /// value index is axis id, value value is (changed, value)
    pub controller_axis: HashMap<GamepadId, HashMap<Axis, (bool, f32)>>,

    /// currently pressed keys
    keys: HashSet<KeyInput>,
    /// keys that were pressed but waiting to be registered
    keys_down: HashSet<(KeyInput, Instant)>,
    /// keys that were released but waiting to be registered
    keys_up: HashSet<(KeyInput, Instant)>,
    
    text_cache: String,
    window_change_focus: Option<bool>,
    register_times: Vec<f32>,

    /// do we try to protect against double taps? if so, whats the duration we should check for?
    pub double_tap_protection: Option<f32>,
    
    /// last key pressed, time it was pressed, was it a double tap? (need to know if it was a double tap for release check)
    last_key_press: HashMap<KeyInput, (Instant, bool)>,
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

            
            controller_info: HashMap::new(),
            controller_buttons: HashMap::new(),
            controller_down: HashMap::new(),
            controller_up: HashMap::new(),
            controller_axis: HashMap::new(),


            text_cache: String::new(),
            window_change_focus: None,

            double_tap_protection: None,
            last_key_press: HashMap::new(),
        }
    }

    
    fn verify_controller_index_exists(&mut self, id: GamepadId, name: Arc<String>, power_info: PowerInfo) {
        if !self.controller_info.contains_key(&id) {
            // window.joystick_deadzone = 0.01;
            debug!("New controller: {}", name);
            self.controller_info.insert(id, GamepadInfo {
                id,
                name,
                power_info,
                connected: true,
            });
        } else {
            return;
        }

        self.controller_buttons.insert(id, HashSet::new());
        self.controller_down.insert(id, HashSet::new());

        self.controller_up.insert(id, HashSet::new());

        let data = [
            Axis::LeftStickX, Axis::LeftStickY, Axis::LeftZ,
            Axis::RightStickX, Axis::RightStickY, Axis::RightZ,
            Axis::DPadX, Axis::DPadY
        ].into_iter().map(|a|(a, (false, 0.0))).collect();

        self.controller_axis.insert(id, data);
    }

    pub fn set_double_tap_protection(&mut self, protection: Option<f32>) {
        self.double_tap_protection = protection;
    }

    pub fn handle_events(&mut self, e:Window2GameEvent) {

        match e {
            // window events
            Window2GameEvent::GotFocus => self.window_change_focus = Some(true),
            Window2GameEvent::LostFocus => {
                self.window_change_focus = Some(false);

                // forcefully release all keys. horrible workaround but its good enough
                for key in std::mem::take(&mut self.keys) {
                    self.keys_up.insert((key, Instant::now()));
                }
            }

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
                    if let winit::keyboard::Key::Character(txt) = &key.logical {
                        self.text_cache += txt;
                    }

                    self.keys.insert(key.clone());
                    self.keys_down.insert((key.clone(), Instant::now()));
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

            Window2GameEvent::ControllerEvent(e, name, power_info) => {
                let id = e.id;

                match e.event {
                    gilrs::EventType::Connected => self.verify_controller_index_exists(id, name, power_info),
                    // gilrs::EventType::Disconnected => todo!(),

                    gilrs::EventType::ButtonPressed(b, _) => {
                        self.controller_down.get_mut(&id).unwrap().insert(b);
                        self.controller_buttons.get_mut(&id).unwrap().insert(b);
                    }
                    gilrs::EventType::ButtonReleased(b, _) => {
                        self.controller_up.get_mut(&id).unwrap().insert(b);
                        self.controller_buttons.get_mut(&id).unwrap().remove(&b);
                    }
                    gilrs::EventType::AxisChanged(a, val, _) => {
                        // info!("controller axis: {a:?} = {val}");
                        *self.controller_axis.get_mut(&id).unwrap().get_mut(&a).unwrap() = (true, val);
                    }


                    // is this like, for ps2 analog buttons?
                    // gilrs::EventType::ButtonChanged(_, _, _) => todo!(),

                    // ignore because it should be ignored
                    // gilrs::EventType::Dropped => todo!(),

                    // cheating (?)
                    // gilrs::EventType::ButtonRepeated(_, _) => todo!(),

                    _ => {}
                }
            }


            _ => {}
        }

    }

    /// is the key currently down (not up)
    pub fn key_down(&self, k:Key) -> bool { self.keys.iter().any(|ki|ki.is_key(k)) }
    pub fn get_key_mods(&self) -> KeyModifiers {
        KeyModifiers {
            ctrl: self.key_down(Key::LControl) || self.key_down(Key::RControl),
            alt: self.key_down(Key::LAlt) || self.key_down(Key::RAlt),
            shift: self.key_down(Key::LShift) || self.key_down(Key::RShift),
        }
    }


    /// get all keys that were pressed, and clear the pressed list. (will be true when first checked and pressed, false after first check or when key is up)
    pub fn get_keys_down(&mut self) -> KeyCollection {
        let mut down = Vec::new();
        for (i, time) in &self.keys_down { down.push(i.clone()); self.register_times.push(time.as_millis()); }
        self.keys_down.clear();

        KeyCollection::new(down)
    }
    pub fn get_keys_up(&mut self) -> KeyCollection {
        let mut up = Vec::new();
        for (i, time) in &self.keys_up { up.push(i.clone()); self.register_times.push(time.as_millis()); }
        self.keys_up.clear();

        KeyCollection::new(up)
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

    pub fn get_controller_info(&self, id: GamepadId) -> Option<GamepadInfo> {
        self.controller_info.get(&id).cloned()
    }


    /// get all pressed controller buttons, and reset the pressed array
    /// (controller_id, button_id)
    pub fn get_controller_down(&mut self) -> Vec<(GamepadInfo, HashSet<ControllerButton>)> {
        // let mut down = Vec::new();
        // for (c, buttons) in self.controller_down.iter_mut() {
        //     let name = self.controller_names.get(c).unwrap();
           
        //     for b in buttons.iter() {
        //         let controller = make_controller(*c, name.clone());
        //         down.push((controller, *b));
        //     }
        //     buttons.clear()
        // }
        // down

        let down = self.controller_down.iter().map(|(g, i)|(self.get_controller_info(*g).unwrap(), i.clone())).collect();
        self.controller_down.iter_mut().for_each(|(_, i)|i.clear());
        down
    }

    /// get all released controller buttons, and reset the pressed array
    /// (controller_id, button_id)
    pub fn get_controller_up(&mut self) -> Vec<(GamepadInfo, HashSet<ControllerButton>)> {
        // let mut up = Vec::new();
        // for (c, buttons) in self.controller_up.iter_mut() {
        //     let name = self.controller_names.get(c).unwrap();
            
        //     for b in buttons.iter() {
        //         // let controller = make_controller(*c, name.clone());
        //         up.push((*c, *b));
        //     }
        //     buttons.clear()
        // }
        // up
        let up = self.controller_up.iter().map(|(g, i)|(self.get_controller_info(*g).unwrap(), i.clone())).collect();
        self.controller_up.iter_mut().for_each(|(_, i)|i.clear());
        up
    }

    /// get all controller axes
    /// (controller, [axis_id, (changed, value)])
    pub fn get_controller_axis(&mut self) -> Vec<(GamepadInfo, HashMap<Axis, (bool, f32)>)> {
        let mut axis = Vec::new();

        for (c, axis_data) in self.controller_axis.iter_mut() {
            // let name = self.controller_names.get(c).unwrap();
            // let controller = make_controller(*c, name.clone());
            // axis.push((controller, axis_data.clone()));
            axis.push((self.controller_info.get(c).cloned().unwrap(), axis_data.clone()));

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
    pub fn get_register_delay(&mut self) -> (f32, f32, f32) {
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


pub struct InputBinding {
    pub keyboard: Option<winit::keyboard::PhysicalKey>,
    pub mouse: Option<winit::event::MouseButton>,
}




#[derive(Clone)]
pub struct GamepadInfo {
    pub id: GamepadId,
    pub name: Arc<String>,
    pub power_info: gilrs::PowerInfo,
    pub connected: bool,
}



#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AxisConfig {
    pub axis_id: Axis,
    pub threshhold: f64
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ControllerInputConfig {
    pub button: Option<ControllerButton>,
    pub axis: Option<AxisConfig>
}
impl ControllerInputConfig {
    pub fn new(button: Option<ControllerButton>, axis: Option<AxisConfig>) -> Self {
        Self {
            button, 
            axis
        }
    }

    pub fn check_button(&self, button: ControllerButton) -> bool {
        if let Some(b) = self.button {
            b == button
        } else {
            false
        }
    }
}

impl From<Axis> for ControllerInputConfig {
    fn from(value: Axis) -> Self {
        Self {
            button: None,
            axis: Some(AxisConfig {axis_id: value, threshhold: 0.0})
        }
    }
}
impl From<ControllerButton> for ControllerInputConfig {
    fn from(value: ControllerButton) -> Self {
        Self {
            button: Some(value),
            axis: None,
        }
    }
}
