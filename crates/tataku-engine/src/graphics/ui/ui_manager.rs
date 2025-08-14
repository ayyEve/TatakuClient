use crate::prelude::*;
use gilrs::{ GamepadId, Axis };

pub struct CurrentInputState {
    pub mouse_pos: Vector2,
    pub mouse_moved: bool,
    pub scroll_delta: Vector2,

    pub mouse_down: Vec<MouseButton>,
    pub mouse_up: Vec<MouseButton>,

    pub keys_down: KeyCollection,
    pub keys_up: KeyCollection,

    pub controller_down: Vec<(ControllerButton, GamepadId, ArcStr)>,
    pub controller_up: Vec<(ControllerButton, GamepadId, ArcStr)>,
    pub controller_axes: Vec<(Axis, f32, GamepadId, ArcStr)>,

    pub mods: KeyModifiers,
}
impl CurrentInputState {
    pub(super) fn make_input(&self, event: InputType) -> InputEvent {
        InputEvent {
            event,
            mouse_pos: self.mouse_pos,
            key_mods: self.mods,
        }
    }

    pub fn into_events(self) -> Vec<InputEvent> {
        [
            self.mouse_moved.then_some(InputType::MouseMove(self.mouse_pos)),
            (self.scroll_delta.x.abs() > f32::EPSILON || self.scroll_delta.y.abs() > f32::EPSILON).then_some(InputType::MouseScroll(self.scroll_delta))
        ]
            .into_iter()
            .flatten()
            .chain(self.mouse_down.into_iter().map(InputType::MousePress))
            .chain(self.mouse_up.into_iter().map(InputType::MouseRelease))
            .chain(self.keys_down.0.into_iter().map(InputType::KeyPress))
            .chain(self.keys_up.0.into_iter().map(InputType::KeyRelease))
            
            .chain(self.controller_down.into_iter().map(|(a, b, c)| InputType::ControllerPress(a, b, c)))
            .chain(self.controller_up.into_iter().map(|(a, b, c)| InputType::ControllerRelease(a, b, c)))
            .chain(self.controller_axes.into_iter().map(|(a, b, c, d)| InputType::ControllerAxis(a, b, c, d)))

            .map(|event| InputEvent { event, mouse_pos: self.mouse_pos, key_mods: self.mods })
            .collect()
    }
}


pub struct GeneralUiTheme {
    pub background_color: Color,
    pub default_color: Color,
    pub hover_color: Color,
    pub active_color: Color,
}
impl GeneralUiTheme {
    pub fn get_color(&self, active: bool, hover: bool) -> Color {
        if active {
            self.active_color
        } else if hover {
            self.hover_color
        } else {
            self.default_color
        }
    }
}
impl Default for GeneralUiTheme {
    fn default() -> Self {
        Self {
            background_color: Color::BLACK.alpha(0.8),
            default_color: Color::WHITE,
            hover_color: Color::CYAN,
            active_color: Color::YELLOW,
        }
    }
}

