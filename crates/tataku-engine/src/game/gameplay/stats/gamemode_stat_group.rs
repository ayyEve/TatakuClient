use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct StatGroup {
    pub name: &'static str,
    pub display_name: &'static str,
    pub stats: &'static [GameModeStat]
}
impl StatGroup {
    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
