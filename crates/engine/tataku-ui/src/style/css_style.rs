use crate::prelude::*;


#[macro_export]
macro_rules! create_css_value {
    ($name: ident, $default: ident; $($str: expr, $variant: ident);* $(;)?) => {
        use $crate::prelude::*;
        #[derive(Deserialize, Reflect)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $($variant),*
        }
        impl Default for $name {
            fn default() -> Self {
                Self::$default
            }
        }
        impl std::str::FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if s == $str { return Ok(Self::$variant) }
                )*

                Err(())
            }
        }
        impl From<$name> for taffy::$name {
            fn from(value: $name) -> Self {
                $(
                    if value == $name::$variant { return taffy::$name::$variant }
                )*
                unreachable!()
            }
        }
    }
}

#[derive(ParseCss)]
#[derive(Default, Debug, Clone)]
pub struct CssStyle {
    /// What layout strategy should be used?
    pub display: CssValue<DisplayType>,

    /// Should size styles apply to the content box or the border box of the node
    pub box_sizing: CssValue<BoxSizing>,

    // Overflow properties
    /// How children overflowing their container should affect layout
    #[css(shorthand = "DualShorthand")] _overflow: (),
    pub overflow_x: CssValue<Overflow>,
    pub overflow_y: CssValue<Overflow>,

    /// How much space (in pixels) should be reserved for the scrollbars of `Overflow::Scroll` and `Overflow::Auto` nodes.
    pub scrollbar_width: CssValue<f32>,

    // Position properties
    /// What should the `position` value of this struct use as a base offset?
    pub position: CssValue<Position>,

    /// How should the position of this element be tweaked relative to the layout defined?
    #[css(shorthand = "QuadShorthand")] _inset: (),
    pub inset_top: CssValue<CssUnit>,
    pub inset_left: CssValue<CssUnit>,
    pub inset_bottom: CssValue<CssUnit>,
    pub inset_right: CssValue<CssUnit>,

    // Size properties
    #[css(shorthand = "DualShorthand", size)] _size: (),
    pub width: CssValue<CssUnit>,
    pub height: CssValue<CssUnit>,

    /// Controls the minimum size of the item
    #[css(shorthand = "DualShorthand", size)] _min_size: (),
    pub min_width: CssValue<CssUnit>,
    pub min_height: CssValue<CssUnit>,

    /// Controls the maximum size of the item
    #[css(shorthand = "DualShorthand", size)] _max_size: (),
    pub max_width: CssValue<CssUnit>,
    pub max_height: CssValue<CssUnit>,

    /// Sets the preferred aspect ratio for the item
    ///
    /// The ratio is calculated as width divided by height.
    pub aspect_ratio: CssValue<f32>,

    // Spacing Properties

    /// How much space (in pixels) should be between items? (currently only used for dropdowns).
    pub item_margin: CssValue<f32>,

    /// How large should the margin be on each side?
    #[css(shorthand = "QuadShorthand")] _margin: (),
    pub margin_top: CssValue<CssUnit>,
    pub margin_left: CssValue<CssUnit>,
    pub margin_bottom: CssValue<CssUnit>,
    pub margin_right: CssValue<CssUnit>,


    /// How large should the padding be on each side?
    #[css(shorthand = "QuadShorthand")] _padding: (),
    pub padding_top: CssValue<CssUnit>,
    pub padding_left: CssValue<CssUnit>,
    pub padding_bottom: CssValue<CssUnit>,
    pub padding_right: CssValue<CssUnit>,


    /// How large should the border be on each side?
    #[css(shorthand = "QuadShorthand")] _border_width: (),
    pub border_width_top: CssValue<CssUnit>,
    pub border_width_left: CssValue<CssUnit>,
    pub border_width_bottom: CssValue<CssUnit>,
    pub border_width_right: CssValue<CssUnit>,

    /// The border radius in px
    pub border_radius: CssValue<f32>,

    /// The border color
    #[css(parse_with = "Self::parse_color")]
    pub border_color: CssValue<Color>,

    /// The background color
    #[css(parse_with = "Self::parse_color")]
    pub background_color: CssValue<Color>,

    // Alignment properties
    /// How this node's children aligned in the cross/block axis?
    pub align_items: CssValue<AlignItems>,

    /// How this node should be aligned in the cross/block axis
    /// Falls back to the parents [`AlignItems`] if not set
    pub align_self: CssValue<AlignSelf>,

    /// How this node's children should be aligned in the inline axis
    pub justify_items: CssValue<AlignItems>,

    /// How this node should be aligned in the inline axis
    /// Falls back to the parents [`JustifyItems`] if not set
    pub justify_self: CssValue<AlignSelf>,

    /// How should content contained within this item be aligned in the cross/block axis
    pub align_content: CssValue<AlignContent>,

    /// How should content contained within this item be aligned in the main/inline axis
    pub justify_content: CssValue<JustifyContent>,

    /// How large should the gaps between items in a grid or flex container be?
    #[css(shorthand = "DualShorthand")] _gap: (),
    pub gap_x: CssValue<CssUnit>,
    pub gap_y: CssValue<CssUnit>,

    // Flexbox container properties
    /// Which direction does the main axis flow in?
    pub flex_direction: CssValue<FlexDirection>,

    /// Should elements wrap, or stay in a single line?
    pub flex_wrap: CssValue<FlexWrap>,

    // Flexbox item properties
    /// Sets the initial main axis size of the item
    pub flex_basis: CssValue<CssUnit>,

    /// The relative rate at which this item grows when it is expanding to fill space
    ///
    /// 0.0 is the default value, and this value must be positive.
    pub flex_grow: CssValue<f32>,

    /// The relative rate at which this item shrinks when it is contracting to fit into space
    ///
    /// 1.0 is the default value, and this value must be positive.
    pub flex_shrink: CssValue<f32>,


    // text properties
    #[css(default = "CssValue::Inherit")]
    #[css(parse_with = "Self::parse_font")]
    pub font: CssValue<DefaultFont>,
    #[css(default = "CssValue::Inherit")]
    pub font_size: CssValue<f32>,
    #[css(default = "CssValue::Inherit")]
    #[css(parse_with = "Self::parse_color")]
    pub text_color: CssValue<Color>,
    #[css(default = "CssValue::Inherit")]
    pub line_height: CssValue<f32>,
    pub text_alignment: CssValue<Alignment>,


    // image properties

    /// What image should be used
    pub image: CssValue<String>,

    /// How should the image be aligned
    pub image_alignment: CssValue<Alignment>,

    /// How should the element fit inside the container
    #[css(parse_with = "Self::parse_image_fit")]
    pub image_stretch: CssValue<ImageStretch>,

    /// Where should the image be loaded from
    #[css(parse_with = "Self::parse_image_source")]
    pub image_source: CssValue<TextureSource>,

    /// Should the image be grayscale
    pub image_grayscale: CssValue<bool>,


    // animation properties

    /// Name of the animation
    pub animation_name: CssValue<String>,
    /// Duration of the animation (in seconds)
    pub animation_duration: CssValue<f32>,
    /// How long to wait before running the animation (in seconds)
    pub animation_delay: CssValue<f32>,

    pub animation_iteration_count: CssValue<AnimationIterationCount>,

    // blur properties
    #[css(shorthand = "BlurShorthand")]
    #[css(shorthand_fields("blur_amount", "blur_type", "blur_location"))]
    _blur: (),

    /// How much to blur, 0 is none
    pub blur_amount: CssValue<f32>,

    /// What type of blur to use, default is box
    pub blur_type: CssValue<CssBlurType>,

    /// Should the blur be applied above or below the element its on (above means it would blur itself)
    pub blur_location: CssValue<BlurLocation>,
}
impl CssStyle {
    pub fn text_style(&self, values: &dyn Reflect) -> TextStyle {
        TextStyle::default()
            .font_maybe(self.font.value().copied())
            .font_size_maybe(self.font_size.resolve_copied(values))
            .color_maybe(self.text_color.resolve_copied(values))
            .line_height_maybe(self.line_height.clone()
                .check_inherit(self.font_size.clone())
                .resolve_copied(values))
            .alignment_maybe(self.text_alignment.value().copied())
    }

    pub fn menu_layout() -> CssStyle {
        let zero = f16::from_f32(0.0);

        CssStyle {
            display: DisplayType::Flex.into(),
            flex_direction: FlexDirection::Column.into(),
            box_sizing: BoxSizing::ContentBox.into(),
            position: Position::Relative.into(),
            overflow_x: Overflow::Hidden.into(),
            overflow_y: Overflow::Hidden.into(),

            align_self: CssValue::Unset,
            align_items: AlignItems::Stretch.into(),
            align_content: AlignContent::SpaceBetween.into(),
            justify_self: CssValue::Unset,
            justify_items: AlignItems::Stretch.into(),
            justify_content: AlignContent::SpaceBetween.into(),

            width: FILL.into(),
            height: FILL.into(),
            min_width: FILL.into(),
            min_height: FILL.into(),
            max_width: FILL.into(),
            max_height: FILL.into(),

            gap_x: CssUnit::Pixels(zero).into(),
            gap_y: CssUnit::Pixels(zero).into(),

            inset_top: CssUnit::Auto.into(),
            inset_left: CssUnit::Auto.into(),
            inset_bottom: CssUnit::Auto.into(),
            inset_right: CssUnit::Auto.into(),

            margin_top: CssUnit::Pixels(zero).into(),
            margin_left: CssUnit::Pixels(zero).into(),
            margin_bottom: CssUnit::Pixels(zero).into(),
            margin_right: CssUnit::Pixels(zero).into(),

            padding_top: CssUnit::Pixels(zero).into(),
            padding_left: CssUnit::Pixels(zero).into(),
            padding_bottom: CssUnit::Pixels(zero).into(),
            padding_right: CssUnit::Pixels(zero).into(),

            border_width_top: CssUnit::Pixels(zero).into(),
            border_width_left: CssUnit::Pixels(zero).into(),
            border_width_bottom: CssUnit::Pixels(zero).into(),
            border_width_right: CssUnit::Pixels(zero).into(),

            aspect_ratio: CssValue::Unset,
            flex_wrap: FlexWrap::NoWrap.into(),
            flex_basis: CssUnit::Auto.into(),
            flex_grow: 0.0f32.into(),
            flex_shrink: 1.0f32.into(),

            ..Default::default()
        }
    }
}



#[test]
fn merge_test() {
    let parent = CssStyle {
        width: CssValue::Value(CssUnit::Pixels(f16::from_f32(100.0))),
        ..Default::default()
    };

    let child = CssStyle {
        width: CssValue::Inherit,
        ..Default::default()
    };


    let merged = child.merge(parent);
    assert_eq!(merged.width, CssValue::Inherit);
}
