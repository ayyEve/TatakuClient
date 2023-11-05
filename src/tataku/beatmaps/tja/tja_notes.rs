
/// a regular taiko note
#[derive(Copy, Clone, Debug)]
pub struct TjaCircle {
    pub time: f32,
    pub is_don: bool,
    pub is_big: bool,
}

#[derive(Copy, Clone, Debug)]
/// a taiko drumroll
pub struct TjaDrumroll {
    pub time: f32,
    pub end_time: f32,
    pub is_big: bool,
}

#[derive(Copy, Clone, Debug)]
/// a taiko spinner
pub struct TjaBalloon {
    pub time: f32,
    pub end_time: f32,
    pub hits_required: usize,
}
