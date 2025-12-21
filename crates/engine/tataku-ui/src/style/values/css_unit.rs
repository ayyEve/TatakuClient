use crate::*;
use std::str::FromStr;
use common::reflect::*;
use core::result::Result;

use taffy::{
    Dimension,
    LengthPercentage,
    LengthPercentageAuto,
};

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum CssUnit<T:Reflect+Copy=f16> {
    /// Automatic
    #[default]
    Auto,
    /// Pixels
    Pixels(T),
    /// Percent of the parent
    Percent(T),

    /// Relative to the font-size of the element 
    Em(T),
    /// Relative to font-size of the root element
    Rem(T),

    /// Relative to 1% of the width of the viewport
    ViewportWidth(T),
    /// Relative to 1% of the height of the viewport
    ViewportHeight(T),

    /// Relative to 1% of viewport's* smaller dimension
    ViewportMin(T),
    /// Relative to 1% of viewport's* larger dimension
    ViewportMax(T),
}

impl<T: IntoF32 + Copy + Reflect> CssUnit<T> {

    // resolve the inner value regardless of variant
    fn resolve_inner(
        self,
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> f32 {
        match self {
            Self::Auto => 0.0,
            Self::Percent(n) => n.into_f32(),
            Self::Pixels(n) => n.into_f32(),

            Self::Em(n) => n.into_f32() * font_size,
            Self::Rem(n) => n.into_f32() * root_font_size,

            Self::ViewportWidth(n) => n.into_f32() * viewport.x * 0.01,
            Self::ViewportHeight(n) => n.into_f32() * viewport.y * 0.01,
            Self::ViewportMin(n) => n.into_f32() * viewport.x.min(viewport.y) * 0.01,
            Self::ViewportMax(n) => n.into_f32() * viewport.x.max(viewport.y) * 0.01,
        }
    }


    pub fn resolve_dimension(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> Dimension {
        let n = self.resolve_inner(viewport, font_size, root_font_size);
        
        match self {
            Self::Auto => Dimension::auto(),
            Self::Percent(_) => Dimension::percent(n.into_f32()),
            _ => Dimension::length(n.into_f32()),
        }
    }
    pub fn resolve_length_percent(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentage {
        let n = self.resolve_inner(viewport, font_size, root_font_size);

        match self {
            Self::Auto => LengthPercentage::percent(1.0),
            Self::Percent(_) => LengthPercentage::percent(n.into_f32()),
            _ => LengthPercentage::length(n.into_f32()),
        }
    }
    pub fn resolve_length_percent_auto(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentageAuto {
        let n = self.resolve_inner(viewport, font_size, root_font_size);
        
        match self {
            Self::Auto => LengthPercentageAuto::auto(),
            Self::Percent(_) => LengthPercentageAuto::percent(n.into_f32()),
            _ => LengthPercentageAuto::length(n.into_f32()),
        }
    }
}

impl<T> FromStr for CssUnit<T> 
where T: FromStr + IntoF32 + std::ops::Div<Output = T> + Copy + Reflect
{
    type Err = CssUnitError<T>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("%") {
            let a = s.trim_end_matches("%")
                .parse::<T>()
                .map_err(Self::Err::Inner)
                ?;
            Ok(Self::Percent(a / T::from_f32(100.0)))
        } else if s.ends_with("px") {
            let a = s.trim_end_matches("px")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::Pixels(a))
        } else if s.ends_with("em") {
            let a = s.trim_end_matches("em")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::Em(a))
        } else if s.ends_with("rem") {
            let a = s.trim_end_matches("rem")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::Rem(a))
        } else if s.ends_with("vw") {
            let a = s.trim_end_matches("vw")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::ViewportWidth(a))
        } else if s.ends_with("vh") {
            let a = s.trim_end_matches("vh")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::ViewportHeight(a))
        } else if s.ends_with("vmin") {
            let a = s.trim_end_matches("vmin")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::ViewportMin(a))
        } else if s.ends_with("vmax") {
            let a = s.trim_end_matches("vmax")
                .parse()
                .map_err(Self::Err::Inner)?;
            Ok(Self::ViewportMax(a))
        }

        else {
            match s {
                "auto" => Ok(Self::Auto),
                "fill" => Ok(Self::Percent(T::from_f32(1.0))),
                _ => Err(Self::Err::UnknownUnit(s.to_owned().into())),
            }
        }
    }
}
impl<T:Reflect + Copy> From<T> for CssUnit<T> {
    fn from(value: T) -> Self {
        Self::Pixels(value)
    }
}
pub enum CssUnitError<T:FromStr> {
    Inner(T::Err),
    UnknownUnit(Cow<'static, str>),
}


pub trait IntoF32 {
    fn into_f32(self) -> f32;
    fn from_f32(n: f32) -> Self;
}
impl IntoF32 for f32 {
    fn into_f32(self) -> f32 { self }
    fn from_f32(n: f32) -> Self { n }
}
impl IntoF32 for f16 {
    fn into_f32(self) -> f32 {
        self.to_f32()
    }
    fn from_f32(n: f32) -> Self {
        f16::from_f32(n)
    }
}
