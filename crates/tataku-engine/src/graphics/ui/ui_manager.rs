use crate::prelude::*;
use crate::prelude::ui::*;
use gilrs::{GamepadId, Axis};

pub struct CurrentInputState {
    pub mouse_pos: Vector2,
    pub mouse_moved: bool,
    pub scroll_delta: f32,

    pub mouse_down: Vec<MouseButton>,
    pub mouse_up: Vec<MouseButton>,

    pub keys_down: KeyCollection,
    pub keys_up: KeyCollection,

    pub controller_down: Vec<(ControllerButton, GamepadId, Arc<String>)>,
    pub controller_up: Vec<(ControllerButton, GamepadId, Arc<String>)>,
    pub controller_axes: Vec<(Axis, f32, GamepadId, Arc<String>)>,

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
            (self.scroll_delta > f32::EPSILON).then_some(InputType::MouseScroll(self.scroll_delta))
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



bitflags::bitflags! {
    #[derive(Copy, Clone, PartialEq, Debug, Default)]
    pub struct ElementState:u8 {
        const None = 0;
        const Hover = 1;
        const Active = 2;
        const Focus = 4;
    }
}



#[derive(Clone, Default)]
pub struct ElementData {
    pub state: ElementState,
    pub element_name: String,
    pub id: Option<String>,
    pub class_list: Vec<String>,

    pub debug_name: Option<String>,

    pub styles: ElementStateStyles<Option<Image>>,
}
impl ElementData {
    pub fn style(&self) -> &(CssStyle, Option<Image>) {
        self.styles.get_style(self.state)
    }
    pub fn style_mut(&mut self) -> &mut (CssStyle, Option<Image>) {
        self.styles.get_style_mut(self.state)
    }
}

#[derive(Clone)]
pub struct TreeData {
    // pub bounds: Bounds,
    pub absolute_bounds: Bounds,
    pub local_transform: Transform,
    pub global_transform: Matrix,
    pub inverse_global_transform: Matrix,
    pub needs_inverse_transform: bool,

    pub selected: Option<bool>,
    pub node_left: Option<TaffyNodeId>,
    pub node_right: Option<TaffyNodeId>,
    pub node_above: Option<TaffyNodeId>,
    pub node_below: Option<TaffyNodeId>,

    pub element_data: ElementData,
}
impl Default for TreeData {
    fn default() -> Self {
        Self { 
            // bounds: Bounds::default(),
            absolute_bounds: Bounds::default(), 
            local_transform: Transform::default(), 
            global_transform: Matrix::identity(), 
            inverse_global_transform: Matrix::identity(), 
            needs_inverse_transform: false,

            selected: None, 
            node_left: None, 
            node_right: None, 
            node_above: None, 
            node_below: None, 

            element_data: ElementData::default(),
        }
    }
}
impl TreeData {
    pub fn selectable(&self) -> bool {
        self.selected.is_some()
    }
    pub fn set_selectable(&mut self, selectable: bool) {
        self.selected = selectable.then_some(false);
    }

    pub fn node_direction(&self, direction: Direction) -> Option<TaffyNodeId> {
        match direction {
            Direction::Up => self.node_above,
            Direction::Down => self.node_below,
            Direction::Left => self.node_left,
            Direction::Right => self.node_right,
        }
    }
}



#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct NodeId {
    pub node_id: TaffyNodeId,
    pub owner: MessageOwner,
}
impl NodeId {
    pub fn new(id: TaffyNodeId, owner: MessageOwner) -> Self {
        Self {
            node_id: id,
            owner,
        }
    }
}
impl Default for NodeId {
    fn default() -> Self { EMPTY_NODE }
}


#[derive(Clone, Debug)]
pub enum MenuType {
    Internal(&'static str),
    Custom(String)
}
#[cfg(feature="graphics")]
impl MenuType {
    pub fn from_menu(menu: &dyn crate::prelude::Widget) -> Self {
        match menu.name() {
            Cow::Borrowed(name) => Self::Internal(name),
            Cow::Owned(name) => Self::Custom(name.clone())
        }
    }
}
