use crate::prelude::*;
use std::str::FromStr;

use taffy::Dimension;
use taffy::LengthPercentage;
use taffy::LengthPercentageAuto;


#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[derive(Reflect)]
pub enum CssUnit<T:Reflect+Copy=half::f16> {
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
impl CssUnit<f32> {
    pub fn resolve_dimension(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> Dimension {
        match self {
            Self::Auto => Dimension::auto(),
            Self::Percent(n) => Dimension::percent(n),
            Self::Pixels(n) => Dimension::length(n),

            Self::Em(n) => Dimension::length(n * font_size),
            Self::Rem(n) => Dimension::length(n * root_font_size),

            Self::ViewportWidth(n) => Dimension::length(n * viewport.x),
            Self::ViewportHeight(n) => Dimension::length(n * viewport.y),
            Self::ViewportMin(n) => Dimension::length(n * viewport.x.min(viewport.y)),
            Self::ViewportMax(n) => Dimension::length(n * viewport.x.max(viewport.y)),
        }
    }
    pub fn resolve_length_percent(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentage {
        match self {
            Self::Auto => LengthPercentage::percent(1.0),
            Self::Percent(n) => LengthPercentage::percent(n),
            Self::Pixels(n) => LengthPercentage::length(n),

            Self::Em(n) => LengthPercentage::length(n * font_size),
            Self::Rem(n) => LengthPercentage::length(n * root_font_size),

            Self::ViewportWidth(n) => LengthPercentage::length(n * viewport.x),
            Self::ViewportHeight(n) => LengthPercentage::length(n * viewport.y),
            Self::ViewportMin(n) => LengthPercentage::length(n * viewport.x.min(viewport.y)),
            Self::ViewportMax(n) => LengthPercentage::length(n * viewport.x.max(viewport.y)),
        }
    }
    pub fn resolve_length_percent_auto(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentageAuto {
        match self {
            Self::Auto => LengthPercentageAuto::auto(),
            Self::Percent(n) => LengthPercentageAuto::percent(n),
            Self::Pixels(n) => LengthPercentageAuto::length(n),

            Self::Em(n) => LengthPercentageAuto::length(n * font_size),
            Self::Rem(n) => LengthPercentageAuto::length(n * root_font_size),

            Self::ViewportWidth(n) => LengthPercentageAuto::length(n * viewport.x),
            Self::ViewportHeight(n) => LengthPercentageAuto::length(n * viewport.y),
            Self::ViewportMin(n) => LengthPercentageAuto::length(n * viewport.x.min(viewport.y)),
            Self::ViewportMax(n) => LengthPercentageAuto::length(n * viewport.x.max(viewport.y)),
        }
    }
}
impl CssUnit<half::f16> {
    pub fn resolve_dimension(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> Dimension {
        match self {
            Self::Auto => Dimension::auto(),
            Self::Percent(n) => Dimension::percent(n.to_f32()),
            Self::Pixels(n) => Dimension::length(n.to_f32()),

            Self::Em(n) => Dimension::length(n.to_f32() * font_size),
            Self::Rem(n) => Dimension::length(n.to_f32() * root_font_size),

            Self::ViewportWidth(n) => Dimension::length(n.to_f32() * viewport.x),
            Self::ViewportHeight(n) => Dimension::length(n.to_f32() * viewport.y),
            Self::ViewportMin(n) => Dimension::length(n.to_f32() * viewport.x.min(viewport.y)),
            Self::ViewportMax(n) => Dimension::length(n.to_f32() * viewport.x.max(viewport.y)),
        }
    }
    pub fn resolve_length_percent(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentage {
        match self {
            Self::Auto => LengthPercentage::percent(1.0),
            Self::Percent(n) => LengthPercentage::percent(n.to_f32()),
            Self::Pixels(n) => LengthPercentage::length(n.to_f32()),

            Self::Em(n) => LengthPercentage::length(n.to_f32() * font_size),
            Self::Rem(n) => LengthPercentage::length(n.to_f32() * root_font_size),

            Self::ViewportWidth(n) => LengthPercentage::length(n.to_f32() * viewport.x),
            Self::ViewportHeight(n) => LengthPercentage::length(n.to_f32() * viewport.y),
            Self::ViewportMin(n) => LengthPercentage::length(n.to_f32() * viewport.x.min(viewport.y)),
            Self::ViewportMax(n) => LengthPercentage::length(n.to_f32() * viewport.x.max(viewport.y)),
        }
    }
    pub fn resolve_length_percent_auto(
        self, 
        viewport: Vector2,
        font_size: f32,
        root_font_size: f32,
    ) -> LengthPercentageAuto {
        match self {
            Self::Auto => LengthPercentageAuto::auto(),
            Self::Percent(n) => LengthPercentageAuto::percent(n.to_f32()),
            Self::Pixels(n) => LengthPercentageAuto::length(n.to_f32()),

            Self::Em(n) => LengthPercentageAuto::length(n.to_f32() * font_size),
            Self::Rem(n) => LengthPercentageAuto::length(n.to_f32() * root_font_size),

            Self::ViewportWidth(n) => LengthPercentageAuto::length(n.to_f32() * viewport.x),
            Self::ViewportHeight(n) => LengthPercentageAuto::length(n.to_f32() * viewport.y),
            Self::ViewportMin(n) => LengthPercentageAuto::length(n.to_f32() * viewport.x.min(viewport.y)),
            Self::ViewportMax(n) => LengthPercentageAuto::length(n.to_f32() * viewport.x.max(viewport.y)),
        }
    }
}

impl FromStr for CssUnit<f32> {
    type Err = CssUnitError<f32>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("%") {
            let a = s.trim_end_matches("%")
                .parse::<f32>()
                .map_err(Self::Err::Inner)
                ?;
            Ok(Self::Percent(a / 100.0))
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
                "fill" => Ok(Self::Percent(1.0)),
                _ => Err(Self::Err::UnknownUnit(s.to_owned().into())),
            }
        }
    }
}

impl FromStr for CssUnit<half::f16> {
    type Err = CssUnitError<half::f16>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with("%") {
            let a = s.trim_end_matches("%")
                .parse::<half::f16>()
                .map_err(Self::Err::Inner)
                ?;
            Ok(Self::Percent(a / half::f16::from_f32(100.0)))
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
                "fill" => Ok(Self::Percent(half::f16::from_f32(1.0))),
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
