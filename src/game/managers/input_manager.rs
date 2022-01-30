use piston::Event;
use piston::TextEvent;
use piston::FocusEvent;
use piston::ButtonEvent;
use piston::input::Button;
use piston::MouseScrollEvent;
use piston::MouseCursorEvent;
use piston::input::ButtonState;
use piston::ControllerAxisEvent;

use crate::prelude::*;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Controller {
    pub id: u32,
    pub name: Arc<String>
}
impl Controller {
    fn new(id:u32, name:Arc<String>) -> Self {
        Self {
            id,
            name
        }
    }
}

// lazy_static::lazy_static! {
//     pub static ref CONTROLLER_NAMES: Arc<Mutex<HashMap<u32, String>>> = Arc::new(Mutex::new(HashMap::new()));
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

            text_cache: String::new(),
            window_change_focus: None,

            raw_input: false
        }
    }

    fn verify_controller_index_exists(&mut self, id: u32, window: &mut glfw_window::GlfwWindow) {
        if !self.controller_names.contains_key(&id) {
            use glfw::JoystickId::*;
            let j_id = match id {
                0  => Joystick1,
                1  => Joystick2,
                2  => Joystick3,
                3  => Joystick4,
                4  => Joystick5,
                5  => Joystick6,
                6  => Joystick7,
                7  => Joystick8,
                8  => Joystick9,
                9  => Joystick10,
                10 => Joystick11,
                11 => Joystick12,
                12 => Joystick13,
                13 => Joystick14,
                14 => Joystick15,
                15 => Joystick16,
                _ => panic!("unknown joystick id: {}", id)
            };

            let name = window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
            println!("New controller: {}", name);
            self.controller_names.insert(id, Arc::new(name));
            // CONTROLLER_NAMES.lock().insert(id, name);
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
    }

    pub fn handle_events(&mut self, e:Event, window:&mut glfw_window::GlfwWindow) {
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

                (Button::Controller(cb), ButtonState::Press) => {
                    println!("press: c: {}, b: {}", cb.id, cb.button);
                    self.verify_controller_index_exists(cb.id, window);
                    self.controller_buttons.get_mut(&cb.id).unwrap().insert(cb.button);
                    self.controller_down.get_mut(&cb.id).unwrap().insert(cb.button);
                }
                (Button::Controller(cb), ButtonState::Release) => {
                    println!("release: c: {}, b: {}", cb.id, cb.button);
                    self.controller_buttons.get_mut(&cb.id).unwrap().remove(&cb.button);
                    self.controller_up.get_mut(&cb.id).unwrap().insert(cb.button);
                }
                _ => {}
            }
        }

        if let Some(axis) = e.controller_axis_args() {
            // println!("got controller axis: {:?}", axis);
        }

        e.mouse_cursor(|[x, y]| {
            let incoming = Vector2::new(x, y);

            if self.raw_input {
                let half_window = Settings::window_size() / 2.0;
                let diff = incoming - half_window;
                if diff == Vector2::zero() {return}
                self.mouse_pos = self.mouse_pos + diff;
                self.mouse_moved = true;

                if !(self.mouse_pos.x < 0.0 || self.mouse_pos.y < 0.0 
                || self.mouse_pos.x > half_window.x * 2.0 || self.mouse_pos.y > half_window.y * 2.0) {
                    window.window.set_cursor_pos(half_window.x, half_window.y)
                }

            } else {
                if incoming == self.mouse_pos {return}
                self.mouse_moved = true;
                self.mouse_pos = incoming;
            }
        });

        e.mouse_scroll(|d| {self.scroll_delta += d[1]});
        if let Some(e) = e.text_args() {self.text_cache += &e}
        if let Some(has_focus) = e.focus_args() {self.window_change_focus = Some(has_focus)}
        // e.text(|text| println!("Typed '{}'", text));
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


    /// get all pressed controller buttons, and reset the pressed array
    /// (controller_id, button_id)
    pub fn get_controller_down(&mut self) -> Vec<(Controller, u8)> {
        let mut down = Vec::new();
        for (c, buttons) in self.controller_down.iter_mut() {
            let name = self.controller_names.get(c).unwrap();
            let controller = Controller::new(*c, name.clone());

            for b in buttons.iter() {
                down.push((controller.clone(), *b));
            }
            buttons.clear()
        }
        down
    }

    /// get all released controller buttons, and reset the pressed array
    /// (controller_id, button_id)
    pub fn get_controller_up(&mut self) -> Vec<(Controller, u8)> {
        let mut up = Vec::new();
        for (c, buttons) in self.controller_up.iter_mut() {
            let name = self.controller_names.get(c).unwrap();
            let controller = Controller::new(*c, name.clone());
            
            for b in buttons.iter() {
                up.push((controller.clone(), *b));
            }
            buttons.clear()
        }
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



#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AxisConfig {
    pub axis_id: u8,
    pub threshhold: f64
}


#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
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


pub fn map_button_to_name(controller: Arc<String>, button: u8) -> &'static str {
    match (&**controller, button) {
        // taiko drum (ps4, switch) (windows | linux)
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 0)  => "Y",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 1)  => "B",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 2)  => "A",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 3)  => "X",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 4)  => "L",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 5)  => "R",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 6)  => "Outer Left",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 7)  => "Outer Right",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 8)  => "Minus",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 9)  => "Plus",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 10) => "Inner Left",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 11) => "Inner Right",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 12) => "Home",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 13) => "Share",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 14) => "D-Pad Up",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 15) => "D-Pad Right",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 16) => "D-Pad Down",
        ("Taiko Controller"|"HORI CO.,LTD. Taiko Controller", 17) => "D-Pad Left",

        // xbox one controller (windows|probably linux)
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 0)  => "A",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 1)  => "B",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 2)  => "X",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 3)  => "Y",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 4)  => "LB",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 5)  => "RB",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 6)  => "Select/Options",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 7)  => "Start",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 8)  => "Left Stick Down",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 9)  => "Right Stick Down",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 10) => "D-Pad Up",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 11) => "D-Pad Right",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 12) => "D-Pad Down",
        ("Xbox Controller"|"Microsoft X-Box One S pad"|"Microsoft X-Box One pad", 13) => "D-Pad Left",

        // ps3 controller (linux only, windows apparently is pain)
        ("Sony PLAYSTATION(R)3 Controller", 0) => "Cross",
        ("Sony PLAYSTATION(R)3 Controller", 1) => "Circle",
        ("Sony PLAYSTATION(R)3 Controller", 2) => "Triangle",
        ("Sony PLAYSTATION(R)3 Controller", 3) => "Square",
        ("Sony PLAYSTATION(R)3 Controller", 4) => "L1",
        ("Sony PLAYSTATION(R)3 Controller", 5) => "R1",
        ("Sony PLAYSTATION(R)3 Controller", 6) => "L2",
        ("Sony PLAYSTATION(R)3 Controller", 7) => "R2",
        ("Sony PLAYSTATION(R)3 Controller", 8) => "Select",
        ("Sony PLAYSTATION(R)3 Controller", 9) => "Start",
        ("Sony PLAYSTATION(R)3 Controller", 10) => "Home",
        ("Sony PLAYSTATION(R)3 Controller", 11) => "L3",
        ("Sony PLAYSTATION(R)3 Controller", 12) => "R3",
        ("Sony PLAYSTATION(R)3 Controller", 13) => "D-Pad Up",
        ("Sony PLAYSTATION(R)3 Controller", 14) => "D-Pad Down",
        ("Sony PLAYSTATION(R)3 Controller", 15) => "D-Pad Left",
        ("Sony PLAYSTATION(R)3 Controller", 16) => "D-Pad Right",

        // ps4 controller (windows | linux)
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 0) => "Square",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 1) => "Cross",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 2) => "Circle",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 3) => "Triangle",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 4) => "L1",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 5) => "R1",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 6) => "L2",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 7) => "R2",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 8) => "Share",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 9) => "Start",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 10) => "L3",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 11) => "R3",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 12) => "Home",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 13) => "Touchpad Click",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 14) => "D-Pad Up",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 15) => "D-Pad Right",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 16) => "D-Pad Down",
        ("Wireless Controller"|"Sony Interactive Entertainment Wireless Controller", 17) => "D-Pad Left",

        // steam controller
        ("Steam Controller", 0)  => "Left Touchpad Touch",
        ("Steam Controller", 1)  => "Right Touchpad Touch",
        ("Steam Controller", 2)  => "A",
        ("Steam Controller", 3)  => "B",
        ("Steam Controller", 4)  => "X",
        ("Steam Controller", 5)  => "Y",
        ("Steam Controller", 6)  => "Left Bumper",
        ("Steam Controller", 7)  => "Right Bumper",
        ("Steam Controller", 8)  => "Left Trigger Hard",
        ("Steam Controller", 27) => "Left Trigger Mid", // should these
        ("Steam Controller", 23) => "Left Trigger Soft", // be swapped?
        ("Steam Controller", 9)  => "Right Trigger Hard",
        ("Steam Controller", 26) => "Right Trigger Mid", // should these
        ("Steam Controller", 22) => "Right Trigger Soft", // be swapped?
        ("Steam Controller", 10) => "Back",
        ("Steam Controller", 11) => "Forward",
        ("Steam Controller", 12) => "Steam Button",
        ("Steam Controller", 13) => "Analog Click",
        ("Steam Controller", 14)  => "Right Touchpad Click",
        ("Steam Controller", 15)  => "Left Back",
        ("Steam Controller", 16)  => "right Back",
        ("Steam Controller", 17) => "D-Pad Up",
        ("Steam Controller", 18) => "D-Pad Down",
        ("Steam Controller", 19) => "D-Pad Right",
        ("Steam Controller", 20) => "D-Pad Left",


        // Mayflash Wiimote PC Adapter
        ("Mayflash Wiimote PC Adapter", 0)  => "1",
        ("Mayflash Wiimote PC Adapter", 1)  => "2",
        ("Mayflash Wiimote PC Adapter", 2)  => "A",
        ("Mayflash Wiimote PC Adapter", 3)  => "B",
        ("Mayflash Wiimote PC Adapter", 4)  => "-",
        ("Mayflash Wiimote PC Adapter", 5)  => "+",
        ("Mayflash Wiimote PC Adapter", 6)  => "Z",
        ("Mayflash Wiimote PC Adapter", 7)  => "C",
        // ("Mayflash Wiimote PC Adapter", 8)  => "?",
        // ("Mayflash Wiimote PC Adapter", 9)  => "?",
        // ("Mayflash Wiimote PC Adapter", 10) => "?",
        ("Mayflash Wiimote PC Adapter", 11) => "Home",
        ("Mayflash Wiimote PC Adapter", 12) => "D-Pad Up",
        ("Mayflash Wiimote PC Adapter", 13) => "D-Pad Right",
        ("Mayflash Wiimote PC Adapter", 14) => "D-Pad Down",
        ("Mayflash Wiimote PC Adapter", 15) => "D-Pad Left",

        // Generic\tUSB\tJoystick\t

        _ => "Other"
    }
}
