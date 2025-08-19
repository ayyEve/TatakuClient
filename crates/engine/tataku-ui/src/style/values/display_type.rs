use crate::prelude::*;

#[derive(Deserialize, Reflect)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum DisplayType {
    #[default] Flex,
    Block,
    // Grid,
    Table,
    None,
}
impl FromStr for DisplayType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "block" => Ok(Self::Block),
            "flex" => Ok(Self::Flex),
            // "grid" => Ok(Self::Grid),
            "table" => Ok(Self::Table),
            "none" => Ok(Self::None),
            _ => Err(())
        }
    }
}
