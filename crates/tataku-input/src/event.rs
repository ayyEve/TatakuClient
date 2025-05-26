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
    MouseScroll(f32),

    ControllerPress(ControllerButton, GamepadId, Arc<String>),
    ControllerRelease(ControllerButton, GamepadId, Arc<String>),
    ControllerAxis(gilrs::Axis, f32, GamepadId, Arc<String>),

    // controller input
    /// really only used by InputManager to handle all controller events
    RawControllerEvent(gilrs::Event, Arc<String>, gilrs::PowerInfo),
}
impl InputType {
    pub fn is_mouse(&self) -> bool {
        matches!(self, Self::MouseMove(_) | Self::MousePress(_) | Self::MouseRelease(_))
    }
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
}