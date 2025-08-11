use std::fmt::Debug;

use crate::prelude::*;
use crate::prelude::ui::*;

use taffy::Size;
use taffy::Rect;
use taffy::Point;
use taffy::TextAlign;
use taffy::Dimension;
use taffy::LengthPercentage;
use taffy::BoxGenerationMode;
use taffy::LengthPercentageAuto;

pub struct CssStyleResolver<'a> {
    pub values: &'a dyn Reflect,
    pub style: &'a CssStyle,

    pub viewport: Vector2,
    pub root_font_size: f32,
}
impl CssStyleResolver<'_> {
    #[inline(always)]
    fn font_size(&self) -> f32 {
        self.style.font_size
            .resolve_copied(self.values)
            .unwrap_or(32.0)
    }

    #[inline(always)]
    fn display(&self) -> DisplayType {
        self.resolve(&self.style.display)
    }
} 

// resolvers
impl CssStyleResolver<'_> {
    #[inline(always)]
    fn resolve_or<T: Copy + Reflect + Debug>(
        &self, 
        value: &CssValue<T>,
        default: T
    ) -> T {
        value
            .resolve_copied(self.values)
            .unwrap_or(default)
    }

    #[inline(always)]
    fn resolve<T: Copy + Reflect + Debug + Default>(
        &self, 
        value: &CssValue<T>
    ) -> T {
        value
            .resolve_copied(self.values)
            .unwrap_or_default()
    }

    #[inline(always)]
    fn resolve_maybe<
        Out,
        In: Copy + Reflect + Debug + Into<Out>,
    >(
        &self, 
        value: &CssValue<In>,
    ) -> Option<Out> {
        value.resolve_copied(self.values)
            .map(|i| i.into())
    }

    #[inline(always)]
    fn resolve_dimension(
        &self, 
        value: &CssValue<CssUnit>,
        default: Dimension,
    ) -> Dimension {
        value.resolve(self.values).map_or(
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
        width: &CssValue<CssUnit>,
        height: &CssValue<CssUnit>,
    ) -> Size<Dimension> {
        Size {
            width: self.resolve_dimension(width, default),
            height: self.resolve_dimension(height, default),
        }
    }

    #[inline(always)]
    fn resolve_lp(
        &self, 
        value: &CssValue<CssUnit>,
        default: LengthPercentage,
    ) -> LengthPercentage {
        value.resolve(self.values).map_or(
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
        top: &CssValue<CssUnit>,
        left: &CssValue<CssUnit>,
        bottom: &CssValue<CssUnit>,
        right: &CssValue<CssUnit>,
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
        width: &CssValue<CssUnit>,
        height: &CssValue<CssUnit>,
    ) -> Size<LengthPercentage> {
        Size {
            width: self.resolve_lp(width, default),
            height: self.resolve_lp(height, default),
        }
    }

    #[inline(always)]
    fn resolve_lpa(
        &self, 
        value: &CssValue<CssUnit>,
        default: LengthPercentageAuto,
    ) -> LengthPercentageAuto {
        value.resolve(self.values).map_or(
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
        top: &CssValue<CssUnit>,
        left: &CssValue<CssUnit>,
        bottom: &CssValue<CssUnit>,
        right: &CssValue<CssUnit>,
    ) -> Rect<LengthPercentageAuto> {
        Rect {
            top: self.resolve_lpa(top, default),
            left: self.resolve_lpa(left, default),
            bottom: self.resolve_lpa(bottom, default),
            right: self.resolve_lpa(right, default),
        }
    }
    
}

impl taffy::CoreStyle for CssStyleResolver<'_> {
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
        self.resolve(&self.style.box_sizing).into()
    }

    fn overflow(&self) -> Point<taffy::Overflow> {
        Point {
            x: self.resolve(&self.style.overflow_x).into(),
            y: self.resolve(&self.style.overflow_y).into(),
        }
    }

    fn scrollbar_width(&self) -> f32 {
        self.resolve(&self.style.scrollbar_width)
    }

    fn position(&self) -> taffy::Position {
        self.resolve(&self.style.position).into()
    }

    fn inset(&self) -> Rect<LengthPercentageAuto> {
        self.resolve_lpa_rect(
            LengthPercentageAuto::auto(), 
            &self.style.inset_top, 
            &self.style.inset_left, 
            &self.style.inset_bottom, 
            &self.style.inset_right,
        )
    }

    fn size(&self) -> Size<Dimension> {
        self.resolve_dimension_size(
            Dimension::auto(), 
            &self.style.width, 
            &self.style.height
        )
    }

    fn min_size(&self) -> Size<Dimension> {
        self.resolve_dimension_size(
            Dimension::auto(), 
            &self.style.min_width, 
            &self.style.min_height,
        )
    }

    fn max_size(&self) -> Size<Dimension> {
        self.resolve_dimension_size(
            Dimension::auto(), 
            &self.style.max_width, 
            &self.style.max_height,
        )
    }

    fn aspect_ratio(&self) -> Option<f32> {
        self.resolve_maybe(&self.style.aspect_ratio)
    }

    fn margin(&self) -> Rect<LengthPercentageAuto> {
        self.resolve_lpa_rect(
            LengthPercentageAuto::length(0.0),
            &self.style.margin_top,
            &self.style.margin_left,
            &self.style.margin_bottom,
            &self.style.margin_right,
        )
    }

    fn padding(&self) -> Rect<LengthPercentage> {
        self.resolve_lp_rect(
            LengthPercentage::length(0.0),
            &self.style.padding_top,
            &self.style.padding_left,
            &self.style.padding_bottom,
            &self.style.padding_right,
        )
    }

    fn border(&self) -> Rect<LengthPercentage> {
        self.resolve_lp_rect(
            LengthPercentage::length(0.0),
            &self.style.border_width_top,
            &self.style.border_width_left,
            &self.style.border_width_bottom,
            &self.style.border_width_right,
        )
    }
}

impl taffy::BlockContainerStyle for CssStyleResolver<'_> {
    fn text_align(&self) -> TextAlign {
        TextAlign::Auto
    }
}
impl taffy::BlockItemStyle for CssStyleResolver<'_> {
    fn is_table(&self) -> bool {
        matches!(self.display(), DisplayType::Table)
    }
}

impl taffy::FlexboxContainerStyle for CssStyleResolver<'_> {
    fn flex_direction(&self) -> taffy::FlexDirection {
        self.resolve(&self.style.flex_direction).into()
    }

    fn flex_wrap(&self) -> taffy::FlexWrap {
        self.resolve(&self.style.flex_wrap).into()
    }

    fn gap(&self) -> Size<LengthPercentage> {
        self.resolve_lp_size(
            LengthPercentage::length(0.0),
            &self.style.gap_x,
            &self.style.gap_y,
        )
    }

    fn align_content(&self) -> Option<taffy::AlignContent> {
        self.resolve_maybe(&self.style.align_content)
    }

    fn align_items(&self) -> Option<taffy::AlignItems> {
        self.resolve_maybe(&self.style.align_items)
    }

    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        self.resolve_maybe(&self.style.justify_content)
    }
}
impl taffy::FlexboxItemStyle for CssStyleResolver<'_> {
    fn flex_basis(&self) -> Dimension {
        self.resolve_dimension(
            &self.style.flex_basis, 
            Dimension::auto()
        )
    }

    fn flex_grow(&self) -> f32 {
        self.resolve(&self.style.flex_grow)
    }

    fn flex_shrink(&self) -> f32 {
        self.resolve_or(&self.style.flex_shrink, 1.0)
    }

    fn align_self(&self) -> Option<taffy::AlignSelf> {
        self.resolve_maybe(&self.style.align_self)
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
