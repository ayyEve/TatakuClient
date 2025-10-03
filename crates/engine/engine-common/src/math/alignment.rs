use crate::prelude::*;
use common::reflect::*;
use common::macros::Reflect;

use HorizontalAlign::*;
use VerticalAlign::*;

#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Center,
    Right,
}
impl HorizontalAlign {
    pub fn resolve(
        self,
        container: &Bounds,
        item_width: f32,
        inside: bool,
    ) -> f32 {
        container.pos.x + match (inside, self) {
            (true, Self::Left) => 0.0,
            (true, Self::Right) => container.size.x - item_width,
            
            // you cant not align something center but not inside lol
            (_, Self::Center) => (container.size.x - item_width) / 2.0,
            
            (false, Self::Left) => -item_width,
            (false, Self::Right) => container.size.x,
        }
    }
}
impl From<Alignment> for HorizontalAlign {
    fn from(value: Alignment) -> Self {
        value.horizontal
    }
}


#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum VerticalAlign {
    #[default]
    Top,
    Middle,
    Bottom,
}
impl VerticalAlign {
    pub fn resolve(
        self,
        container: &Bounds,
        item_height: f32,
        inside: bool,
    ) -> f32 {
        container.pos.y + match (inside, self) {
            (true, Self::Top) => 0.0,
            (true, Self::Bottom) => container.size.y - item_height,

            (_, Self::Middle) => (container.size.y - item_height) / 2.0,

            (false, Self::Top) => -item_height,
            (false, Self::Bottom) => container.size.y,
        }
    }
}
impl From<Alignment> for VerticalAlign {
    fn from(value: Alignment) -> Self {
        value.vertical
    }
}


#[derive(Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Alignment {
    pub horizontal: HorizontalAlign,
    pub vertical: VerticalAlign,
}
impl Alignment {
    pub const TOP_LEFT:Self = Self::new(Left, Top);
    pub const TOP_CENTER:Self = Self::new(Center, Top);
    pub const TOP_RIGHT:Self = Self::new(Right, Top);

    pub const CENTER_LEFT:Self = Self::new(Left, Middle);
    pub const CENTER:Self = Self::new(Center, Middle);
    pub const CENTER_RIGHT:Self = Self::new(Right, Middle);

    pub const BOTTOM_LEFT:Self = Self::new(Left, Bottom);
    pub const BOTTOM_MIDDLE:Self = Self::new(Center, Bottom);
    pub const BOTTOM_RIGHT:Self = Self::new(Right, Bottom);

    pub const fn new(horizontal: HorizontalAlign, vertical: VerticalAlign) -> Self {
        Self {
            horizontal,
            vertical
        }
    }

    pub fn resolve(
        self,
        container: &Bounds,
        item_size: Vector2,
        h_inside: bool,
        v_inside: bool
    ) -> Vector2 {
        Vector2::new(
            self.horizontal.resolve(container, item_size.x, h_inside),
            self.vertical.resolve(container, item_size.y, v_inside)
        )
    }
}
impl From<(VerticalAlign, HorizontalAlign)> for Alignment {
    fn from((vertical, horizontal): (VerticalAlign, HorizontalAlign)) -> Self {
        Self {
            horizontal,
            vertical
        }
    }
}

impl From<(HorizontalAlign, VerticalAlign)> for Alignment {
    fn from((horizontal, vertical): (HorizontalAlign, VerticalAlign)) -> Self {
        Self {
            horizontal,
            vertical
        }
    }
}


impl std::str::FromStr for Alignment {
    type Err = &'static str;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "top-left" | "top_left" | "topleft" => Ok(Self::TOP_LEFT),
            "top" | "top-center" | "top_center" | "topcenter" | "top-middle" | "top_middle" | "topmiddle" => Ok(Self::TOP_CENTER),
            "top-right" | "top_right" | "topright" => Ok(Self::TOP_RIGHT),

            "left" | "center-left" | "center_left" | "centerleft" => Ok(Self::CENTER_LEFT),
            "center" => Ok(Self::CENTER),
            "right" | "center-right" |  "center_right" | "centerright" => Ok(Self::CENTER_RIGHT),

            "bottom-left" | "bottom_left" | "bottomleft" => Ok(Self::BOTTOM_LEFT),
            "bottom"  | "bottom-center" | "bottom_center" | "bottomcenter" | "bottom_middle" | "bottommiddle" => Ok(Self::BOTTOM_MIDDLE),
            "bottom-right" | "bottom_right" | "bottomright" => Ok(Self::BOTTOM_RIGHT),
            
            _ => Err("Unknown Value")
        }
    }
}
