bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ElementState:u8 {
        const None = 0;
        const Hover = 1;
        const Active = 2;
        const Focus = 4;
    }
}