use crate::*;
use common::reflect::*;

#[allow(unused)]
#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[reflect(display="display")]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum GroupBy {
    #[default]
    Set,
    Collections,
}
impl GroupBy {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Set,
            Self::Collections,
        ]
    }
}
impl std::fmt::Display for GroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
