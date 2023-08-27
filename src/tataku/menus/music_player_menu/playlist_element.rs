use crate::prelude::*;

pub struct PlaylistElement {
    pub hidden: bool,
    
    list: ScrollableArea,
}


pub struct PlaylistItem {
    pos: Vector2,
    size: Vector2,
    tag: String,
    
}
