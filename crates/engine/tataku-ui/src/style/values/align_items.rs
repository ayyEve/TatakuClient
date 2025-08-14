crate::create_css_value!(
    AlignItems, Start; 
    "start", Start;
    "end", End;
    "flex-start", FlexStart;
    "flex-end", FlexEnd;
    "center", Center;
    "baseline", Baseline;
    "stretch", Stretch;
);
pub type AlignSelf = AlignItems;
