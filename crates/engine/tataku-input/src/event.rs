use crate::*;

#[derive(Clone, PartialEq, Debug)]
pub enum InputType {
    // keyboard input
    KeyPress(KeyInput),
    KeyRelease(KeyInput),

    // mouse input
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseMove(Vector2),
    MouseScroll(ScrollInput),

    ControllerPress(GamepadButton, GamepadId, ArcStr),
    ControllerRelease(GamepadButton, GamepadId, ArcStr),
    ControllerAxis(gilrs::Axis, f32, GamepadId, ArcStr),

    // controller input
    /// really only used by InputManager to handle all controller events
    RawControllerEvent(gilrs::Event, ArcStr, gilrs::PowerInfo),
}

#[derive(Clone, PartialEq, Debug)]
pub struct InputEvent {
    pub event: InputType,
    pub mouse_pos: Vector2,
    pub key_mods: KeyModifiers,
}
impl InputEvent {
    pub fn is_mouse(&self) -> bool {
        matches!(
            self.event, 
            InputType::MouseMove(_) 
            | InputType::MousePress(_) 
            | InputType::MouseRelease(_) 
            | InputType::MouseScroll {..}
        )
    }
    pub fn is_keyboard(&self) -> bool {
        matches!(
            self.event,
            InputType::KeyPress(_)
            | InputType::KeyRelease(_)
        )
    }

    pub fn is_gamepad(&self) -> bool {
        matches!(
            self.event,
            InputType::ControllerPress(_,_,_)
            | InputType::ControllerRelease(_,_,_)
            | InputType::ControllerAxis(_,_,_,_)
        )
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ScrollInput {
    pub sensitivity: f32,
    pub value: ScrollType
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ScrollType {
    Pixels(Vector2),
    Lines([i32; 2]),
}
impl ScrollInput {
    pub fn get_delta(self, pixels_per_line: f32) -> Vector2 {
        match self.value {
            ScrollType::Pixels(p) => Vector2::new(
                p.x,
                p.y * self.sensitivity
            ),
            ScrollType::Lines([
                x, 
                y
            ]) => Vector2::new(
                x as f32, 
                y as f32 * self.sensitivity * pixels_per_line
            ),
        }
    }
}
