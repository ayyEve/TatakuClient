use crate::prelude::*;

pub struct Discord;
impl Discord {
    pub fn new() -> TatakuResult<Self> {Err(TatakuError::String("Discord not built".to_owned()))}
    pub fn change_status(&mut self, _desc:String) {}
}