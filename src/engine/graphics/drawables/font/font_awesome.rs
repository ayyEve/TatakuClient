
/// list of points for font awesome font
#[repr(u32)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FontAwesome {
    Lock = 0xf023,
    UnlockAlt = 0xf13e,

    Backward = 0xf04a,
    Play = 0xf04b,
    Pause = 0xf04c,
    Stop = 0xf04d,
    Forward = 0xf04e,

    Backward_Step = 0xf048,
    Forward_Step = 0xf051,

    Circle_Pause = 0xf28b,
    Circle_Play = 0xf144,
    Circle_Stop = 0xf28d,

    ArrowPointer = 0xf245,
    HandPointer = 0xf25a,
    UpDown = 0xf338,
    LeftRight = 0xf337,
    UpDownLeftRight = 0xf0b2,
    ICursor = 0xf246,

    WindowMaximize = 0xf2d0,
    WindowMinimize = 0xf2d1,
    WindowRestore = 0xf2d2,
    WindowClose = 0xf2d3,
    WindowCloseOutline = 0xf2d4,


    Crown = 0xf521,
}
impl FontAwesome {
    pub fn get_char(&self) -> char {
        let c = *self as u32;
        char::from_u32(c).expect(&format!("invalid char: {c:#06x}"))
    }
}

impl ToString for FontAwesome {
    fn to_string(&self) -> String {
        self.get_char().to_string()
    }
}