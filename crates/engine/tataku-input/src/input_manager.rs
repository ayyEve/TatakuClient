// use gilrs::PowerInfo;
use crate::*;

#[derive(Default)]
pub struct InputManager {
    pub mouse_pos: Vector2,

    pub using_controller_input: bool,
    pub controller_cursor_pos: Vector2,

    pub events: Vec<InputType>,

    /// currently pressed keys, internal use only
    keys: HashSet<KeyInput>,
    key_mods: KeyModifiers,
    
    text_cache: String,
    window_change_focus: Option<bool>,
    register_times: Vec<f32>,

    /// do we try to protect against double taps? if so, whats the duration we should check for?
    pub double_tap_protection: Option<f32>,
    pub controller_menu_button_config: ControllerButtonMenuConfig,
    
    /// last key pressed, time it was pressed, was it a double tap? (need to know if it was a double tap for release check)
    last_key_press: HashMap<KeyInput, (Instant, bool)>,
}
impl InputManager {
    // fn verify_controller_index_exists(
    //     &mut self, 
    //     id: GamepadId, 
    //     name: ArcStr, 
    //     power_info: PowerInfo
    // ) {
    //     if self.controllers.contains_key(&id) {
    //         return;
    //     }

    //     // window.joystick_deadzone = 0.01;
    //     debug!("New controller: {name}");
    //     self.controllers.insert(id, GamepadState::new(GamepadInfo {
    //         id,
    //         name,
    //         power_info,
    //         connected: true,
    //     }));
    // }

    pub fn set_double_tap_protection(&mut self, protection: Option<f32>) {
        self.double_tap_protection = protection;
    }

    fn map_mods(k: Key) -> Option<KeyModifiers> {
        match k {
            Key::LAlt | Key::RAlt => Some(KeyModifiers::ALT),
            Key::LControl | Key::RControl => Some(KeyModifiers::CTRL),
            Key::LShift | Key::RShift => Some(KeyModifiers::SHIFT),
            _ => None
        }
    }

    pub fn handle_input(&mut self, e: InputType) {
        use InputType as Input;

        match &e {
            // keyboard input
            Input::KeyPress(input) if !self.keys.contains(input) => {
                let mut ok_to_continue = true;

                if let Some(check) = self.double_tap_protection && let Some((
                    press_time, 
                    is_double_tap
                )) = self.last_key_press.get_mut(input) {
                    let since = press_time.as_millis();
                    if since <= check {
                        warn!("stopped a doubletap of duration {since:.4}ms");
                        ok_to_continue = false;
                        *is_double_tap = false;
                    }
                }

                if !ok_to_continue { return }

                if let Some(txt) = &input.text {
                    self.text_cache += txt;
                }

                self.keys.insert(input.clone());

                if let Some(k) = input.key 
                && let Some(m) = Self::map_mods(k) {
                    self.key_mods.insert(m);
                }

                // self.keys_down.insert((key.clone(), TatakuInstant::now()));
                // self.last_key_press.insert(key, (TatakuInstant::now(), false));
            }
            Input::KeyRelease(input) => {
                let mut ok_to_continue = true;

                if self.double_tap_protection.is_some()
                && let Some((_, is_double_tap)) = self.last_key_press.get(input)
                && *is_double_tap {
                    ok_to_continue = false;
                }
                
                if ok_to_continue {
                    self.keys.remove(input);
                    // self.keys_up.insert((key, TatakuInstant::now()));
                    // self.last_key_press.remove(&key);

                    if let Some(k) = input.key 
                    && let Some(m) = Self::map_mods(k) {
                        self.key_mods.remove(m);
                    }
                } else {
                    self.last_key_press.remove(input);
                    return;
                }
            }


            // // mouse input
            // Input::MousePress(mb) => {
            //     self.mouse_buttons.insert(mb);
            //     self.mouse_down.insert((mb, TatakuInstant::now()));
            // }
            // Input::MouseRelease(mb) => {
            //     self.mouse_buttons.remove(&mb);
            //     self.mouse_up.insert((mb, TatakuInstant::now()));
            // }
            // // Input::MouseMove(mouse_pos) => {
            // //     if mouse_pos == self.mouse_pos { return }
            // //     self.mouse_moved = true;
            // //     self.mouse_pos = mouse_pos;
            // // }
            // Input::MouseScroll(delta) => self.scroll_delta += delta,

            Input::RawControllerEvent(event, name, _power_info) => {
                // self.verify_controller_index_exists(event.id, name, power_info);
                // let Some(controller) = self.controllers.get_mut(&event.id) 
                // else { return };

                match event.event {
                    gilrs::EventType::ButtonPressed(b, c) => {
                        self.events.push(InputType::ControllerPress((b, c).into(), event.id, name.clone()));

                        // let b = b.into();
                        // controller.buttons_down.insert(b);
                        // controller.buttons.insert(b);
                    }
                    gilrs::EventType::ButtonReleased(b, c) => {
                        self.events.push(InputType::ControllerRelease((b, c).into(), event.id, name.clone()));
                        // let b = b.into();
                        // controller.buttons_up.insert(b);
                        // controller.buttons.remove(&b);
                    }
                    gilrs::EventType::AxisChanged(axis, val, _) => {
                        self.events.push(InputType::ControllerAxis(axis, val, event.id, name.clone()));
                        // // info!("controller axis: {a:?} = {val}");
                        // if let Some(state) = controller.axis.get_mut(&axis) {
                        //     state.changed = true;
                        //     state.value = val;
                        // }
                    }

                    // is this like, for ps2 analog buttons?
                    // gilrs::EventType::ButtonChanged(_, _, _) => todo!(),
                    
                    // cheating (?)
                    // gilrs::EventType::ButtonRepeated(_, _) => todo!(),

                    _ => {}
                }
            }
        
            _ => {},
        }

        self.events.push(e);
    }

    pub fn set_window_focus(&mut self, focus: bool) {
        if self.window_change_focus == Some(focus) { return }
        self.window_change_focus = Some(focus);

        if !focus {
            // forcefully release all keys. horrible workaround but its good enough
            for key in std::mem::take(&mut self.keys) {
                // self.keys_up.insert((key, TatakuInstant::now()));
                self.events.push(InputType::KeyRelease(key));
            }
        }
    }

    /// is the key currently down (not up)
    // pub fn key_down(&self, k:Key) -> bool { self.keys.iter().any(|ki|ki.is_key(k)) }
    pub fn get_key_mods(&self) -> KeyModifiers {
        self.key_mods
        // KeyModifiers {
        //     ctrl: self.key_down(Key::LControl) || self.key_down(Key::RControl),
        //     alt: self.key_down(Key::LAlt) || self.key_down(Key::RAlt),
        //     shift: self.key_down(Key::LShift) || self.key_down(Key::RShift),
        // }
    }


    // /// get all keys that were pressed, and clear the pressed list. (will be true when first checked and pressed, false after first check or when key is up)
    // pub fn get_keys_down(&mut self) -> KeyCollection {
    //     let mut down = Vec::new();
    //     for (i, time) in &self.keys_down { down.push(i.clone()); self.register_times.push(time.as_millis()); }
    //     self.keys_down.clear();

    //     KeyCollection::new(down)
    // }
    // pub fn get_keys_up(&mut self) -> KeyCollection {
    //     let mut up = Vec::new();
    //     for (i, time) in &self.keys_up { up.push(i.clone()); self.register_times.push(time.as_millis()); }
    //     self.keys_up.clear();

    //     KeyCollection::new(up)
    // }


    // /// get all pressed mouse buttons, and reset the pressed array
    // pub fn get_mouse_down(&mut self) -> Vec<MouseButton> {
    //     let mut down = Vec::new();
    //     for (i, time) in &self.mouse_down { down.push(*i); self.register_times.push(time.as_millis()); }
    //     self.mouse_down.clear();
    //     down
    // }
    // pub fn get_mouse_up(&mut self) -> Vec<MouseButton> {
    //     let mut up = Vec::new();
    //     for (i, time) in &self.mouse_up { up.push(*i); self.register_times.push(time.as_millis()); }
    //     self.mouse_up.clear();
    //     up
    // }

    // /// get whether the mouse was moved or not
    // pub fn get_mouse_moved(&mut self) -> bool {
    //     std::mem::take(&mut self.mouse_moved)
    // }
    // /// get how much the mouse wheel as scrolled (vertically) since the last check
    // pub fn get_scroll_delta(&mut self) -> Vector2 {
    //     std::mem::take(&mut self.scroll_delta)
    // }

    // pub fn get_controller_info(&self, id: GamepadId) -> Option<GamepadInfo> {
    //     Some(self.controllers.get(&id)?.info.clone())
    // }


    // /// get all pressed controller buttons, and reset the pressed array
    // /// (controller_id, button_id)
    // pub fn get_controller_down(&mut self) -> Vec<(GamepadInfo, HashSet<ControllerButton>)> {
    //     self.controllers.values_mut()
    //         .map(|c| (c.info.clone(), std::mem::take(&mut c.buttons_down)))
    //         .collect()
    // }

    // /// get all released controller buttons, and reset the pressed array
    // /// (controller_id, button_id)
    // pub fn get_controller_up(&mut self) -> Vec<(GamepadInfo, HashSet<ControllerButton>)> {
    //     self.controllers.values_mut()
    //         .map(|c| (c.info.clone(), std::mem::take(&mut c.buttons_up)))
    //         .collect()
    // }

    // /// get all controller axes
    // /// (controller, [axis_id, (changed, value)])
    // pub fn get_controller_axis(&mut self) -> Vec<(GamepadInfo, HashMap<Axis, AxisState>)> {
    //     let mut axes = Vec::new();

    //     for controller in self.controllers.values_mut() {
    //         axes.push((controller.info.clone(), controller.axis.clone()));
    //         controller.axis.values_mut().for_each(|a| a.changed = false);
    //     }

    //     axes
    // }
    
    /// gets any text typed since the last check
    pub fn get_text(&mut self) -> String {
        self.text_cache.take()
    }

    /// get whether the window's focus has changed
    pub fn get_changed_focus(&mut self) -> Option<bool> {
        self.window_change_focus.take()
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
    pub keyboard: Option<Key>,
    pub mouse: Option<MouseButton>,
    pub controller: Option<ControllerInputBinding>,
}

// TODO: rename lol
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
pub enum ControllerButtonMenuConfig {
    /// Standard menu enter and menu back buttons
    /// ie, on playstation, circle = back and x = enter
    #[default]
    Standard,
    
    /// Swap the menu back and menu enter buttons
    /// ie, on playstation, circle = enter and x = back
    Japanese,
}
impl ControllerButtonMenuConfig {
    pub fn is_standard(self) -> bool {
        matches!(self, Self::Standard)
    }
}
