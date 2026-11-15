use crate::*;
use common::reflect::*;

pub type JustifyContent = AlignContent;

#[derive(Deserialize, Reflect)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AlignContent {
    #[default] 
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}
impl std::str::FromStr for AlignContent {
    type Err = ();
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            "flex-start" => Ok(Self::FlexStart),
            "flex-end" => Ok(Self::FlexEnd),
            "center" => Ok(Self::Center),
            "stretch" => Ok(Self::Stretch),
            "space-between" => Ok(Self::SpaceBetween),
            "space-evenly" => Ok(Self::SpaceEvenly),
            "space-around" => Ok(Self::SpaceAround),
            _ => Err(())
        }
    }
}

impl From<AlignContent> for taffy::AlignContent {
    fn from(value: AlignContent) -> Self {
        let keyword = match value {
            AlignContent::Start => taffy::AlignContentKeyword::Start,
            AlignContent::End => taffy::AlignContentKeyword::End,
            AlignContent::FlexStart => taffy::AlignContentKeyword::FlexStart,
            AlignContent::FlexEnd => taffy::AlignContentKeyword::FlexEnd,
            AlignContent::Center => taffy::AlignContentKeyword::Center,
            AlignContent::Stretch => taffy::AlignContentKeyword::Stretch,
            AlignContent::SpaceBetween => taffy::AlignContentKeyword::SpaceBetween,
            AlignContent::SpaceEvenly => taffy::AlignContentKeyword::SpaceEvenly,
            AlignContent::SpaceAround => taffy::AlignContentKeyword::SpaceAround,
        };

        taffy::AlignContent {
            keyword,
            safety: taffy::AlignmentSafety::Default
        }
    }
}