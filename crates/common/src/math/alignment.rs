
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}


#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Align {
    TopLeft, TopMiddle, TopRight,
    CenterLeft, CenterMiddle, CenterRight,
    BottomLeft, BottomMiddle, BottomRight,
}
