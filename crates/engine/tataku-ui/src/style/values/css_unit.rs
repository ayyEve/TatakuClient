use crate::*;
use std::str::FromStr;
use common::reflect::*;
use core::result::Result;

use taffy::{
    Dimension,
    CompactLength,
    LengthPercentage,
    LengthPercentageAuto,
};

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum CssUnit<T:Reflect+Copy=f16> {
    /// Automatic
    #[default]
    Auto,

    /// Minimum Content
    MinContent,
    /// Maximum Content
    MaxContent,
    /// Fit Content
    FitContent,


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
    fn resolve_inner<T2>(
        self,
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,

        from_raw: fn(CompactLength) -> T2
    ) -> Result<f32, T2> {
        match self {
            Self::Auto => Ok(0.0),
            Self::MinContent => Err(from_raw(CompactLength::fit_content_percent(1.0))),
            Self::MaxContent => Err(from_raw(CompactLength::min_content())),
            Self::FitContent => Err(from_raw(CompactLength::max_content())),

            Self::Percent(n) => Ok(n.into_f32()),
            Self::Pixels(n) => Ok(n.into_f32()),

            Self::Em(n) => Ok(n.into_f32() * font_size),
            Self::Rem(n) => Ok(n.into_f32() * root_font_size),

            Self::ViewportWidth(n) => Ok(n.into_f32() * viewport.x * 0.01),
            Self::ViewportHeight(n) => Ok(n.into_f32() * viewport.y * 0.01),
            Self::ViewportMin(n) => Ok(n.into_f32() * viewport.x.min(viewport.y) * 0.01),
            Self::ViewportMax(n) => Ok(n.into_f32() * viewport.x.max(viewport.y) * 0.01),
        }
    }


    pub fn resolve_dimension(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> Dimension {
        let n = self.resolve_inner(
            viewport, 
            font_size, 
            root_font_size, 
            |a| unsafe { Dimension::from_raw(a) }
        );

        match (n, self) {
            (Err(n), _) => n,
            (Ok(_), Self::Auto) => Dimension::auto(),
            (Ok(n), Self::Percent(_)) => Dimension::percent(n.into_f32()),
            (Ok(n), _) => Dimension::length(n.into_f32()),
        }
    }
    pub fn resolve_length_percent(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentage {
        let n = self.resolve_inner(
            viewport, 
            font_size, 
            root_font_size, 
            |a| unsafe { LengthPercentage::from_raw(a) }
        );

        match (n, self) {
            (Err(n), _) => n,
            (Ok(_), Self::Auto) => LengthPercentage::percent(1.0),
            (Ok(n), Self::Percent(_)) => LengthPercentage::percent(n.into_f32()),
            (Ok(n), _) => LengthPercentage::length(n.into_f32()),
        }
    }
    pub fn resolve_length_percent_auto(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentageAuto {
        let n = self.resolve_inner(
            viewport, 
            font_size, 
            root_font_size,
            |a| unsafe { LengthPercentageAuto::from_raw(a) }
        );
        
        match (n, self) {
            (Err(n), _) => n,
            (Ok(_), Self::Auto) => LengthPercentageAuto::auto(),
            (Ok(n), Self::Percent(_)) => LengthPercentageAuto::percent(n.into_f32()),
            (Ok(n), _) => LengthPercentageAuto::length(n.into_f32()),
        }
    }
}

impl<T: FromStr + IntoF32> FromStr for CssUnit<T> {
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
                "min-content" => Ok(Self::MinContent),
                "max-content" => Ok(Self::MaxContent),
                "fit-content" => Ok(Self::FitContent),

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


pub trait IntoF32: Copy + std::ops::Div<Output = Self> + Reflect  {
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
