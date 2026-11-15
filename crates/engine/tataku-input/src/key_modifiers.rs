use tataku_engine_common::common::bitflags;

bitflags! {
    #[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
    pub struct KeyModifiers: u8 {
        // Constants are always `pub`
        const CTRL = 1;
        const ALT = 2;
        const SHIFT = 3;
    }
}

