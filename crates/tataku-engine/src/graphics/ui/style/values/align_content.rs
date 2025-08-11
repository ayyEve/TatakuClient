crate::create_css_value!(
    AlignContent, Start; 
    "start", Start;
    "end", End;
    "flex-start", FlexStart;
    "flex-end", FlexEnd;
    "center", Center;
    "stretch", Stretch;
    "space-between", SpaceBetween;
    "space-evenly", SpaceEvenly;
    "space-around", SpaceAround;
);
pub type JustifyContent = AlignContent;
