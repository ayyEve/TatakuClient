use tataku_engine_common::prelude::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ElementState:u8 {
        const None = 0;
        const Hover = 1 << 1;
        const Active = 1 << 2;
        const Focus = 1 << 3;
        const Pressed = 1 << 4;
    }
}

impl ElementState {
    pub const fn list() -> &'static [Self] {
        &[
            Self::None,
            Self::Hover,
            Self::Active,
            Self::Focus,
            Self::Pressed,
        ]
    }

    pub fn hover(self) -> bool { self.contains(Self::Hover) }
    pub fn focus(self) -> bool { self.contains(Self::Focus) }
    pub fn active(self) -> bool { self.contains(Self::Active) }
    pub fn pressed(self) -> bool { self.contains(Self::Pressed) }

    pub fn set_hover(&mut self, val: bool) { self.set(Self::Hover, val) }
    pub fn set_focus(&mut self, val: bool) { self.set(Self::Focus, val) }
    pub fn set_active(&mut self, val: bool) { self.set(Self::Active, val) }
    pub fn set_pressed(&mut self, val: bool) { self.set(Self::Pressed, val) }
}
