use tataku_client_common::prelude::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ElementState:u8 {
        const None = 0;
        const Hover = 1;
        const Active = 2;
        const Focus = 4;
    }
}

impl ElementState {
    pub const fn list() -> &'static [Self] {
        &[
            Self::None,
            Self::Hover,
            Self::Active,
            Self::Focus
        ]
    }
}