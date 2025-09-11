use crate::*;
use common::reflect::*;

#[repr(C)]
#[derive(Reflect)]
#[derive(Copy, Clone, Debug)]
pub struct StatGroup {
    pub name: &'static str,
    pub display_name: &'static str,
    pub stats: &'static [ gameplay::stats::GameModeStat ]
}
impl StatGroup {
    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
