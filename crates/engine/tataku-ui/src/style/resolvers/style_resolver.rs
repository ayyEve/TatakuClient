use crate::*;
use std::fmt::Debug;
use crate::style::css::*;
use common::reflect::*;

use taffy::Size;
use taffy::Rect;
use taffy::Point;
use taffy::TextAlign;
use taffy::Dimension;
use taffy::LengthPercentage;
use taffy::BoxGenerationMode;
use taffy::LengthPercentageAuto;

pub struct NodeStyleResolver<'a> {
    pub values: &'a dyn Reflect,
    pub style: &'a Style,

    pub viewport: Vector2,
    pub root_font_size: f32,
}
impl NodeStyleResolver<'_> {
    #[inline(always)]
    fn font_size(&self) -> f32 {
        self.resolve_or(CssProperty::FontSize, 32.0)
    }

    #[inline(always)]
    fn display(&self) -> DisplayType {
        self.resolve(CssProperty::Display)
    }
} 

// resolvers
impl NodeStyleResolver<'_> {
    #[inline(always)]
    fn resolve_maybe<T: Copy + Reflect + Debug>(
        &self, 
        property: CssProperty,
    ) -> Option<T> {
        self.style.get(property)
            .and_then(|p| p.value().resolve_copied(self.values))
    }

    #[inline(always)]
    fn resolve_or<T: Copy + Reflect + Debug>(
        &self, 
        property: CssProperty,
        default: T
    ) -> T {
        self.resolve_maybe(property).unwrap_or(default)
    }

    #[inline(always)]
    fn resolve<T: Copy + Reflect + Debug + Default>(
        &self, 
        property: CssProperty,
    ) -> T {
        self.resolve_maybe(property).unwrap_or_default()
    }

    #[inline(always)]
    fn resolve_into<
        In: Copy + Reflect + Debug + Into<Out>,
        Out,
    >(
        &self, 
        property: CssProperty,
    ) -> Option<Out> {
        self.resolve_maybe::<In>(property)
            .map(|i| i.into())
    }

    #[inline(always)]
    fn resolve_dimension(
        &self, 
        property: CssProperty,
        value_override: Option<CssUnit>,
        default: Dimension,
    ) -> Dimension {
        value_override
        .or_else(|| self.resolve_maybe(property))
        .map_or(
            default, 
            |i| i.resolve_dimension(
                self.viewport, 
                self.font_size(), 
                self.root_font_size
            )
        )
    }
    
    #[inline(always)]
    fn resolve_dimension_size(
        &self, 
        default: Dimension,
        width: CssProperty,
        width_override: Option<CssUnit>,

        height: CssProperty,
        height_override: Option<CssUnit>,
    ) -> Size<Dimension> {
        Size {
            width: self.resolve_dimension(width, width_override, default),
            height: self.resolve_dimension(height, height_override, default),
        }
    }

    #[inline(always)]
    fn resolve_lp(
        &self, 
        property: CssProperty,
        default: LengthPercentage,
    ) -> LengthPercentage {
        self.resolve_maybe::<CssUnit>(property).map_or(
            default, 
            |i| i.resolve_length_percent(
                self.viewport, 
                self.font_size(), 
                self.root_font_size
            )
        )
    }
    
    #[inline(always)]
    fn resolve_lp_rect(
        &self,
        default: LengthPercentage,
        top: CssProperty,
        left: CssProperty,
        bottom: CssProperty,
        right: CssProperty,
    ) -> Rect<LengthPercentage> {
        Rect {
            top: self.resolve_lp(top, default),
            left: self.resolve_lp(left, default),
            bottom: self.resolve_lp(bottom, default),
            right: self.resolve_lp(right, default),
        }
    }

    #[inline(always)]
    fn resolve_lp_size(
        &self, 
        default: LengthPercentage,
        width: CssProperty,
        height: CssProperty,
    ) -> Size<LengthPercentage> {
        Size {
            width: self.resolve_lp(width, default),
            height: self.resolve_lp(height, default),
        }
    }

    #[inline(always)]
    fn resolve_lpa(
        &self, 
        property: CssProperty,
        default: LengthPercentageAuto,
    ) -> LengthPercentageAuto {
        self.resolve_maybe::<CssUnit>(property).map_or(
            default, 
            |i| i.resolve_length_percent_auto(
                self.viewport, 
                self.font_size(), 
                self.root_font_size
            )
        )
    }

    #[inline(always)]
    fn resolve_lpa_rect(
        &self,
        default: LengthPercentageAuto,
        top: CssProperty,
        left: CssProperty,
        bottom: CssProperty,
        right: CssProperty,
    ) -> Rect<LengthPercentageAuto> {
        Rect {
            top: self.resolve_lpa(top, default),
            left: self.resolve_lpa(left, default),
            bottom: self.resolve_lpa(bottom, default),
            right: self.resolve_lpa(right, default),
        }
    }
    
}

impl taffy::CoreStyle for NodeStyleResolver<'_> {
    type CustomIdent = Arc<str>;

    fn box_generation_mode(&self) -> BoxGenerationMode {
        match self.display() {
            DisplayType::None => BoxGenerationMode::None,
            _ => BoxGenerationMode::Normal,
        }
    }

    fn is_block(&self) -> bool {
        matches!(self.display(), DisplayType::Block)
    }

    fn is_compressible_replaced(&self) -> bool {
        false
    }

    fn box_sizing(&self) -> taffy::BoxSizing {
        self.resolve::<values::BoxSizing>(CssProperty::BoxSizing).into()
    }

    fn overflow(&self) -> Point<taffy::Overflow> {
        Point {
            x: self.resolve::<values::Overflow>(CssProperty::OverflowX).into(),
            y: self.resolve::<values::Overflow>(CssProperty::OverflowY).into(),
        }
    }

    fn scrollbar_width(&self) -> f32 {
        self.resolve(CssProperty::ScrollbarWidth)
    }

    fn position(&self) -> taffy::Position {
        self.resolve::<values::Position>(CssProperty::Position).into()
    }

    fn inset(&self) -> Rect<LengthPercentageAuto> {
        self.resolve_lpa_rect(
            LengthPercentageAuto::auto(), 
            CssProperty::InsetTop, 
            CssProperty::InsetLeft, 
            CssProperty::InsetBottom, 
            CssProperty::InsetRight,
        )
    }

    fn size(&self) -> Size<Dimension> {
        self.resolve_dimension_size(
            Dimension::auto(), 
            CssProperty::Width, 
            None,
            CssProperty::Height,
            None,
        )
    }

    fn min_size(&self) -> Size<Dimension> {
        self.resolve_dimension_size(
            Dimension::auto(), 
            CssProperty::MinWidth, 
            None,
            CssProperty::MinHeight,
            None,
        )
    }

    fn max_size(&self) -> Size<Dimension> {
        self.resolve_dimension_size(
            Dimension::auto(), 
            CssProperty::MaxWidth, 
            None,
            CssProperty::MaxHeight,
            None,
        )
    }

    fn aspect_ratio(&self) -> Option<f32> {
        self.resolve(CssProperty::AspectRatio)
    }

    fn margin(&self) -> Rect<LengthPercentageAuto> {
        self.resolve_lpa_rect(
            LengthPercentageAuto::length(0.0),
            CssProperty::MarginTop,
            CssProperty::MarginLeft,
            CssProperty::MarginBottom,
            CssProperty::MarginRight,
        )
    }

    fn padding(&self) -> Rect<LengthPercentage> {
        self.resolve_lp_rect(
            LengthPercentage::length(0.0),
            CssProperty::PaddingTop,
            CssProperty::PaddingLeft,
            CssProperty::PaddingBottom,
            CssProperty::PaddingRight,
        )
    }

    fn border(&self) -> Rect<LengthPercentage> {
        self.resolve_lp_rect(
            LengthPercentage::length(0.0),
            CssProperty::BorderWidthTop,
            CssProperty::BorderWidthLeft,
            CssProperty::BorderWidthBottom,
            CssProperty::BorderWidthRight,
        )
    }
}

impl taffy::BlockContainerStyle for NodeStyleResolver<'_> {
    fn text_align(&self) -> TextAlign {
        TextAlign::Auto
    }
}
impl taffy::BlockItemStyle for NodeStyleResolver<'_> {
    fn is_table(&self) -> bool {
        matches!(self.display(), DisplayType::Table)
    }
}

impl taffy::FlexboxContainerStyle for NodeStyleResolver<'_> {
    fn flex_direction(&self) -> taffy::FlexDirection {
        self.resolve::<values::FlexDirection>(CssProperty::FlexDirection).into()
    }

    fn flex_wrap(&self) -> taffy::FlexWrap {
        self.resolve::<values::FlexWrap>(CssProperty::FlexWrap).into()
    }

    fn gap(&self) -> Size<LengthPercentage> {
        self.resolve_lp_size(
            LengthPercentage::length(0.0),
            CssProperty::GapX,
            CssProperty::GapY,
        )
    }

    fn align_content(&self) -> Option<taffy::AlignContent> {
        self.resolve_into::<values::AlignContent, _>(CssProperty::AlignContent)
    }

    fn align_items(&self) -> Option<taffy::AlignItems> {
        self.resolve_into::<values::AlignItems, _>(CssProperty::AlignItems)
    }

    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        self.resolve_into::<values::JustifyContent, _>(CssProperty::JustifyContent)
    }
}
impl taffy::FlexboxItemStyle for NodeStyleResolver<'_> {
    fn flex_basis(&self) -> Dimension {
        self.resolve_dimension(
            CssProperty::FlexBasis, 
            None,
            Dimension::auto()
        )
    }

    fn flex_grow(&self) -> f32 {
        self.resolve_or(CssProperty::FlexGrow, 0.0)
    }

    fn flex_shrink(&self) -> f32 {
        self.resolve_or(CssProperty::FlexShrink, 1.0)
    }

    fn align_self(&self) -> Option<taffy::AlignSelf> {
        self.resolve_into::<values::AlignSelf, _>(CssProperty::AlignSelf)
    }
}

// impl taffy::GridContainerStyle for CssStyleResolver<'_> {
//     type Repetition<'a>
//     where Self: 'a;

//     type TemplateTrackList<'a>
//     where Self: 'a;

//     type AutoTrackList<'a>
//     where Self: 'a;

//     type TemplateLineNames<'a>
//     where Self: 'a;

//     type GridTemplateAreas<'a>
//     where Self: 'a;

//     fn grid_template_rows(&self) -> Option<Self::TemplateTrackList<'_>> {
//         todo!()
//     }

//     fn grid_template_columns(&self) -> Option<Self::TemplateTrackList<'_>> {
//         todo!()
//     }

//     fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
//         todo!()
//     }

//     fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
//         todo!()
//     }

//     fn grid_template_areas(&self) -> Option<Self::GridTemplateAreas<'_>> {
//         todo!()
//     }

//     fn grid_template_column_names(&self) -> Option<Self::TemplateLineNames<'_>> {
//         todo!()
//     }

//     fn grid_template_row_names(&self) -> Option<Self::TemplateLineNames<'_>> {
//         todo!()
//     }
// }
// impl taffy::GridItemStyle for CssStyleResolver<'_> {
//     #[inline(always)]
//     fn align_self(&self) -> Option<taffy::AlignSelf> {
//         self.resolve_maybe(&self.style.align_self)
//             .map(|i| i.into())
//     }

//     #[inline(always)]
//     fn justify_self(&self) -> Option<taffy::AlignSelf> {
//         self.resolve_maybe(&self.style.justify_self)
//             .map(|i| i.into())
//     }
    
//     fn grid_row(&self) -> taffy::Line<taffy::GridPlacement<Self::CustomIdent>> {
//         Default::default()
//     }
    
//     fn grid_column(&self) -> taffy::Line<taffy::GridPlacement<Self::CustomIdent>> {
//         Default::default()
//     }
    
//     fn grid_placement(&self, axis: taffy::AbsoluteAxis) -> taffy::Line<taffy::GridPlacement<Self::CustomIdent>> {
//         match axis {
//             taffy::AbsoluteAxis::Horizontal => self.grid_column(),
//             taffy::AbsoluteAxis::Vertical => self.grid_row(),
//         }
//     }
// }
