use crate::*;
use common::reflect::*;

pub type AlignSelf = AlignItems;

#[derive(Deserialize, Reflect)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AlignItems {
    #[default] 
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}
impl std::str::FromStr for AlignItems {
    type Err = ();
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            "flex-start" => Ok(Self::FlexStart),
            "flex-end" => Ok(Self::FlexEnd),
            "center" => Ok(Self::Center),
            "stretch" => Ok(Self::Stretch),
            "baseline" => Ok(Self::Baseline),
            _ => Err(())
        }
    }
}

impl From<AlignItems> for taffy::AlignItems {
    fn from(value: AlignItems) -> Self {
        let keyword = match value {
            AlignItems::Start => taffy::AlignItemsKeyword::Start,
            AlignItems::End => taffy::AlignItemsKeyword::End,
            AlignItems::FlexStart => taffy::AlignItemsKeyword::FlexStart,
            AlignItems::FlexEnd => taffy::AlignItemsKeyword::FlexEnd,
            AlignItems::Center => taffy::AlignItemsKeyword::Center,
            AlignItems::Stretch => taffy::AlignItemsKeyword::Stretch,
            AlignItems::Baseline => taffy::AlignItemsKeyword::Baseline,
        };

        taffy::AlignItems {
            keyword,
            safety: taffy::AlignmentSafety::Default
        }
    }
}