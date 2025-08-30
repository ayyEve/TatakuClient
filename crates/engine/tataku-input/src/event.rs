use crate::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum InputType {
    // keyboard input
    KeyPress(KeyInput),
    KeyRelease(KeyInput),

    // mouse input
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseMove(Vector2),
    MouseScroll(Vector2),

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
            | InputType::MouseScroll(_)
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
