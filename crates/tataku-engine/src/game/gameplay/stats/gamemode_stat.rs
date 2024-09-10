// use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameModeStat {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
}
impl GameModeStat {
    pub const DEFAULT:Self = Self {
        name: "",
        display_name: "",
        description: ""
    };

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
impl AsRef<str> for GameModeStat {
    fn as_ref(&self) -> &str {
        self.name
    }
}

impl Eq for GameModeStat {}
impl PartialEq for GameModeStat {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl std::hash::Hash for GameModeStat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl Default for GameModeStat {
    fn default() -> Self {
        Self::DEFAULT
    }
}
