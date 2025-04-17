use taffy::*;
use crate::prelude::*;
use crate::prelude::ui::*;

macro_rules! impl_parse {
    ($fn: ident, $struct: ident, $(($i: expr, $v: tt));*) => {
        pub fn $fn(s: &str) -> Result<$struct, ()> {
            match s {
                $( $i => Ok($struct::$v), )*
                _ => Err(())
            }
        }
    };

    (rect, $fn: ident, $parent_fn: ident :: $parent_fn2: ident, $struct: ident) => {
        pub fn $fn(s: &str) -> Result<Rect<$struct>, ()> {
            let a = s.trim()
                .split(" ")
                .map($parent_fn :: $parent_fn2)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| ())?;
            
            match a.len() {
                0 => Err(()),
                1 => { 
                    Ok(Rect {
                        top: a[0],
                        left: a[0],
                        bottom: a[0],
                        right: a[0],
                    }) 
                }
                2 => { 
                    Ok(Rect {
                        top: a[0],
                        left: a[1],
                        bottom: a[0],
                        right: a[1]
                    }) 
                }
                4 => {
                    Ok(Rect {
                        top: a[0],
                        left: a[1],
                        bottom: a[2],
                        right: a[3]
                    }) 
                }
                _ => Err(())
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[derive(Deserialize)]
pub enum DisplayType {
    Block,
    #[default] Flex,
    Grid,
    Table,
    None,
}
impl std::str::FromStr for DisplayType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "block" => Ok(Self::Block),
            "flex" => Ok(Self::Flex),
            "grid" => Ok(Self::Grid),
            "table" => Ok(Self::Table),
            "none" => Ok(Self::None),
            _ => Err(())
        }
    }
}


#[derive(Copy, Clone, Debug, Default)]
pub enum CssValue<T> {
    #[default]
    Unset,
    Inherit,
    Value(T),
}
impl<T> CssValue<T> {
    pub fn parse<E>(
        s: &str, 
        default: Self,
        value_parser: impl Fn(&str) -> Result<T, E>
    ) -> Self {
        match s {
            "unset" => Self::Unset,
            "inherit" => Self::Inherit,

            other => value_parser(other)
                .map(Self::Value)
                .unwrap_or(default),
        }
    }

    pub fn check_inherit(self, parent: Self) -> Self {
        match (self, parent) { 
            (Self::Inherit, value @ Self::Value(_)) => value,
            (other, _) => other
        }
    }
    pub fn check_unset(self, parent: Self) -> Self {
        match (self, parent) { 
            (Self::Inherit | Self::Unset, value @ Self::Value(_)) => value,
            (other, _) => other
        }
    }
    
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Value(v) => Some(v),
            _ => None
        }
    }
}


/// this is mostly the same as taffy::Style but everything is an Option.
#[derive(Default, Debug, Clone)]
#[derive(ParseCss)]
pub struct CssStyle {
    #[css(name = "display")]
    /// What layout strategy should be used?
    pub display: CssValue<DisplayType>,

    /// Should size styles apply to the content box or the border box of the node
    #[css(parse_with = "Self::parse_box_sizing")]
    pub box_sizing: CssValue<BoxSizing>,

    // Overflow properties
    /// How children overflowing their container should affect layout
    #[css(parse_with = "Self::parse_overflow")]
    pub overflow_x: CssValue<Overflow>,
    #[css(parse_with = "Self::parse_overflow")]
    pub overflow_y: CssValue<Overflow>,
    /// How much space (in points) should be reserved for the scrollbars of `Overflow::Scroll` and `Overflow::Auto` nodes.
    pub scrollbar_width: CssValue<f32>,

    // Position properties
    /// What should the `position` value of this struct use as a base offset?
    #[css(parse_with = "Self::parse_position")]
    pub position: CssValue<Position>,
    
    /// How should the position of this element be tweaked relative to the layout defined?
    #[css(parse_with = "Self::parse_rect_length_percentage_auto")]
    pub inset: CssValue<Rect<LengthPercentageAuto>>,

    // Size properties
    #[css(parse_with = "Self::parse_dimension")]
    pub width: CssValue<Dimension>,
    #[css(parse_with = "Self::parse_dimension")]
    pub height: CssValue<Dimension>,

    /// Controls the minimum size of the item
    #[css(parse_with = "Self::parse_dimension")]
    pub min_width: CssValue<Dimension>,
    #[css(parse_with = "Self::parse_dimension")]
    pub min_height: CssValue<Dimension>,

    /// Controls the maximum size of the item
    #[css(parse_with = "Self::parse_dimension")]
    pub max_width: CssValue<Dimension>,
    #[css(parse_with = "Self::parse_dimension")]
    pub max_height: CssValue<Dimension>,

    /// Sets the preferred aspect ratio for the item
    ///
    /// The ratio is calculated as width divided by height.
    pub aspect_ratio: CssValue<f32>,

    // Spacing Properties
    /// How large should the margin be on each side?
    #[css(parse_with = "Self::parse_rect_length_percentage_auto")]
    pub margin: CssValue<Rect<LengthPercentageAuto>>,
    /// How large should the padding be on each side?
    #[css(parse_with = "Self::parse_rect_length_percentage")]
    pub padding: CssValue<Rect<LengthPercentage>>,
    /// How large should the border be on each side?
    #[css(parse_with = "Self::parse_rect_length_percentage")]
    pub border: CssValue<Rect<LengthPercentage>>,

    /// The border radius in px
    pub border_radius: CssValue<f32>,

    /// The border color
    #[css(parse_with = "Self::parse_color")]
    pub border_color: CssValue<Color>,

    /// The background color
    #[css(parse_with = "Self::parse_color")]
    pub background: CssValue<Color>,

    // Alignment properties
    /// How this node's children aligned in the cross/block axis?
    #[css(parse_with = "Self::parse_align_items")]
    pub align_items: CssValue<AlignItems>,

    /// How this node should be aligned in the cross/block axis
    /// Falls back to the parents [`AlignItems`] if not set
    #[css(parse_with = "Self::parse_align_self")]
    pub align_self: CssValue<AlignSelf>,

    /// How this node's children should be aligned in the inline axis
    #[css(name="justify-items")]
    #[css(parse_with = "Self::parse_align_items")]
    pub justify_items: CssValue<AlignItems>,

    /// How this node should be aligned in the inline axis
    /// Falls back to the parents [`JustifyItems`] if not set
    #[css(parse_with = "Self::parse_align_self")]
    pub justify_self: CssValue<AlignSelf>,

    /// How should content contained within this item be aligned in the cross/block axis
    #[css(parse_with = "Self::parse_align_content")]
    pub align_content: CssValue<AlignContent>,

    /// How should content contained within this item be aligned in the main/inline axis
    #[css(parse_with = "Self::parse_align_content")]
    pub justify_content: CssValue<JustifyContent>,

    /// How large should the gaps between items in a grid or flex container be?
    #[css(parse_with = "Self::parse_length_percentage")]
    pub gap_x: CssValue<LengthPercentage>,
    #[css(parse_with = "Self::parse_length_percentage")]
    pub gap_y: CssValue<LengthPercentage>,


    // Flexbox container properties
    /// Which direction does the main axis flow in?
    #[css(parse_with = "Self::parse_flex_direction")]
    pub flex_direction: CssValue<FlexDirection>,

    /// Should elements wrap, or stay in a single line?
    #[css(parse_with = "Self::parse_flex_wrap")]
    pub flex_wrap: CssValue<FlexWrap>,

    // Flexbox item properties
    /// Sets the initial main axis size of the item
    #[css(parse_with = "Self::parse_dimension")]
    pub flex_basis: CssValue<Dimension>,

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
    pub font: CssValue<Font>,
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
    pub image_fit: CssValue<ImageFit>,

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


    /// How much to blur, 0 is none
    pub blur: CssValue<f32>,

    /// Should the blur be applied above or below the element its on (above means it would blur itself)
    pub blur_location: CssValue<BlurLocation>,


    // TODO: figure out the best way to parse this
    // // Grid container properies
    // /// Defines the track sizing functions (heights) of the grid rows
    // pub grid_template_rows: Option<Vec<TrackSizingFunction>>,

    // /// Defines the track sizing functions (widths) of the grid columns
    // pub grid_template_columns: Option<Vec<TrackSizingFunction>>,

    // /// Defines the size of implicitly created rows
    // pub grid_auto_rows: Option<Vec<NonRepeatedTrackSizingFunction>>,

    // /// Defined the size of implicitly created columns
    // pub grid_auto_columns: Option<Vec<NonRepeatedTrackSizingFunction>>,

    // /// Controls how items get placed into the grid for auto-placed items
    // pub grid_auto_flow: Option<GridAutoFlow>,

    // // Grid child properties
    // /// Defines which row in the grid the item should start and end at
    // pub grid_row: Option<taffy::Line<GridPlacement>>,

    // /// Defines which column in the grid the item should start and end at
    // pub grid_column: Option<taffy::Line<GridPlacement>>,
}

impl CssStyle {
    pub fn taffy_style(&self) -> taffy::Style {
        let mut default = taffy::Style::default();

        macro_rules! cmp {
            ($($field: ident),*) => {
                $(
                    if let Some(a) = self.$field.value().cloned() {
                        default.$field = a;
                    }
                )*
            };
            (option; $($field: ident),*) => {
                $(default.$field = self.$field.value().cloned();)*
            };
        }

        if let Some(display) = self.display.value() {
            match display {
                DisplayType::Block => default.display = taffy::Display::Block,
                DisplayType::Flex => default.display = taffy::Display::Flex,
                DisplayType::Grid => default.display = taffy::Display::Grid,
                DisplayType::None => default.display = taffy::Display::None,
                DisplayType::Table => default.item_is_table = true,
            }
        }

        if let Some(&x) = self.overflow_x.value() {
            default.overflow.x = x;
        }
        if let Some(&y) = self.overflow_y.value() {
            default.overflow.y = y;
        }
        if let Some(&w) = self.gap_x.value() {
            default.gap.width = w;
        }
        if let Some(&h) = self.gap_y.value() {
            default.gap.height = h;
        }
        
        if let Some(&w) = self.width.value() {
            default.size.width = w;
        }
        if let Some(&h) = self.height.value() {
            default.size.height = h;
        }
        if let Some(&w) = self.min_width.value() {
            default.min_size.width = w;
        }
        if let Some(&h) = self.min_height.value() {
            default.min_size.height = h;
        }
        if let Some(&w) = self.max_width.value() {
            default.max_size.width = w;
        }
        if let Some(&h) = self.max_height.value() {
            default.max_size.height = h;
        }


        cmp!(
            box_sizing,
            scrollbar_width,
            position,
            inset,
            margin,
            padding,
            border,
            flex_direction,
            flex_wrap,
            flex_grow,
            flex_shrink,
            flex_basis
        );
        cmp!(option;
            aspect_ratio,
            align_items,
            align_self,
            justify_items,
            justify_self,
            align_content,
            justify_content
        );
        

        default
    }

    pub fn text_style(&self) -> TextStyle {
        TextStyle::default()
            .font_maybe(self.font.value().copied())
            .font_size_maybe(self.font_size.value().copied())
            .color_maybe(self.text_color.value().copied())
            .line_height_maybe(self.line_height.check_inherit(self.font_size).value().copied())
            .alignment_maybe(self.text_alignment.value().copied())
    }
}

// parsing
#[allow(clippy::result_unit_err)] // its fine because im lazy and we dont care about the error
impl CssStyle {
    pub fn parse_length_percentage_auto(s: &str) -> Result<LengthPercentageAuto, ()> {
        if s.ends_with("%") {
            let a = s.trim_end_matches("%").parse::<f32>().map_err(|_| ())?;
            return Ok(LengthPercentageAuto::Percent(a / 100.0))
        }
        if s.ends_with("px") {
            let a = s.trim_end_matches("px").parse().map_err(|_| ())?;
            return Ok(LengthPercentageAuto::Length(a))
        }

        match s {
            "auto" => Ok(LengthPercentageAuto::Auto),
            _ => Err(())
        }
    }
    pub fn parse_length_percentage(s: &str) -> Result<LengthPercentage, ()> {
        if s.ends_with("%") {
            let a = s.trim_end_matches("%").parse::<f32>().map_err(|_| ())?;
            return Ok(LengthPercentage::Percent(a / 100.0))
        }
        if s.ends_with("px") {
            let a = s.trim_end_matches("px").parse().map_err(|_| ())?;
            return Ok(LengthPercentage::Length(a))
        }

        Err(())
    }

    pub fn parse_dimension(s: &str) -> Result<Dimension, ()> {
        if s.ends_with("%") {
            let a = s.trim_end_matches("%").parse::<f32>().map_err(|_| ())?;
            return Ok(Dimension::Percent(a / 100.0))
        }
        if s.ends_with("px") {
            let a = s.trim_end_matches("px").parse().map_err(|_| ())?;
            return Ok(Dimension::Length(a))
        }

        match s {
            "fill" => Ok(Dimension::Percent(1.0)),
            "auto" => Ok(Dimension::Auto),
            _ => Err(())
        }
    }

    pub fn parse_color(s: &str) -> Result<Color, ()> {
        Color::try_from_hex(s).ok_or(())
    }

    pub fn parse_image_source(s: &str) -> Result<TextureSource, ()> {
        match s {
            "raw" => Ok(TextureSource::Raw),
            "skin" => Ok(TextureSource::Skin),
            "default-skin" => Ok(TextureSource::DefaultSkin),
            other => Ok(TextureSource::Beatmap(other.to_owned()))
        }
    }

    impl_parse!(rect, 
        parse_rect_length_percentage, 
        Self::parse_length_percentage, 
        LengthPercentage
    );
    impl_parse!(rect, 
        parse_rect_length_percentage_auto, 
        Self::parse_length_percentage_auto, 
        LengthPercentageAuto
    );
    impl_parse!(rect, 
        parse_rect_f32, 
        str::parse, 
        f32
    );

    
    impl_parse!(
        parse_image_fit, ImageFit, 
        ("fill", Fill);
        ("none", None);
        ("cover", Cover);
        ("contain", Contain)
    );

    impl_parse!(
        parse_box_sizing, BoxSizing, 
        ("border-box", BorderBox);
        ("content-box", ContentBox)
    );
    impl_parse!(
        parse_overflow, Overflow, 
        ("visible", Visible);
        ("clip", Clip);
        ("hidden", Hidden);
        ("scroll", Scroll)
    );

    impl_parse!(
        parse_position, Position, 
        ("relative", Relative);
        ("absolute", Absolute)
    );

    impl_parse!(
        parse_flex_wrap, FlexWrap, 
        ("nowrap", NoWrap);
        ("wrap", Wrap);
        ("wrap-reverse", WrapReverse)
    );

    impl_parse!(
        parse_flex_direction, FlexDirection, 
        ("row", Row);
        ("column", Column);
        ("row-reverse", RowReverse);
        ("column-reverse", ColumnReverse)
    );
    
    impl_parse!(
        parse_font, Font, 
        ("main", Main);
        ("font-awesome", FontAwesome);
        ("icon", FontAwesome);
        ("icons", FontAwesome);
        ("fallback", Fallback)
    );
    impl_parse!(
        parse_align_items, AlignItems, 
        ("start", Start);
        ("end", End);
        ("center", Center);
        ("flex-start", FlexStart);
        ("flex-end", FlexEnd);
        ("stretch", Stretch);
        ("baseline", Baseline)
    );
    impl_parse!(
        parse_align_self, AlignSelf, 
        ("start", Start);
        ("end", End);
        ("center", Center);
        ("flex-start", FlexStart);
        ("flex-end", FlexEnd);
        ("stretch", Stretch);
        ("baseline", Baseline)
    );
    impl_parse!(
        parse_align_content, AlignContent, 
        ("start", Start);
        ("end", End);
        ("center", Center);
        ("flex-start", FlexStart);
        ("flex-end", FlexEnd);
        ("stretch", Stretch);
        ("space-around", SpaceAround);
        ("space-event", SpaceEvenly);
        ("space-between", SpaceBetween)
    );
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnimationIterationCount {
    Value(u32),
    Infinite
}
impl std::str::FromStr for AnimationIterationCount {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_lowercase() {
            "infinite" => Ok(Self::Infinite),

            other => other.parse().map_err(|_| ()),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AnimationDirection {
    /// The animation is played as normal (forwards). This is default
    #[default]
    Normal,
    /// The animation is played in reverse direction (backwards)
    Reverse,
    /// The animation is played forwards first, then backwards
    Alternate,
    /// The animation is played backwards first, then forwards
    AlternateReverse,
}

impl std::str::FromStr for AnimationDirection {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "reverse" => Ok(Self::Reverse),
            "alternate" => Ok(Self::Alternate),
            "alternate-reverse" => Ok(Self::AlternateReverse),
            _ => Err(()),
        }
    }
}


#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum AnimationTimingFunction {
    /// Specifies an animation with the same speed from start to end
    Linear,

    /// Specifies an animation with a slow start, then fast, then end slowly
    #[default]
    Ease,

    /// Specifies an animation with a slow start
    EaseIn,

    /// Specifies an animation with a slow end
    EaseOut,

    /// Specifies an animation with a slow start and end
    EaseInOut, 
    
    /// Lets you define your own values in a cubic-bezier function
    CubicBezier(u32, u32, u32, u32),
}
impl std::str::FromStr for AnimationTimingFunction {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linear" => Ok(Self::Linear),
            "ease" => Ok(Self::Ease),
            "ease-in" => Ok(Self::EaseIn),
            "ease-out" => Ok(Self::EaseOut),
            "ease-in-out" => Ok(Self::EaseInOut),
            other if other.starts_with("cubic-bezier") => {
                let mut parser = CssValueParser::new(other.trim_start_matches("cubic-bezier"));
                parser.skip_spaces();

                parser.advance(1); // skip the opening (
                parser.skip_spaces(); // skip spaces between ( and first number
                let n1 = parser.read_until(|c| c == ',').trim(); // read first value
                parser.skip_spaces(); // skip spaces between values
                let n2 = parser.read_until(|c| c == ',').trim(); // read value
                parser.skip_spaces(); // skip spaces between values
                let n3 = parser.read_until(|c| c == ',').trim(); // read value
                parser.skip_spaces(); // skip spaces between values
                let n4 = parser.read_until(|c| c == ')').trim(); // read value
                Ok(Self::CubicBezier(
                    n1.parse().map_err(|_| ())?,
                    n2.parse().map_err(|_| ())?,
                    n3.parse().map_err(|_| ())?,
                    n4.parse().map_err(|_| ())?,
                ))
            },

            _ => Err(()),
        }
    }
}


#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum BlurLocation {
    Above,
    #[default]
    Below,
}
impl std::str::FromStr for BlurLocation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "above" => Ok(Self::Above),
            "below" => Ok(Self::Below),
            _ => Err(()),
        }
    }
}


struct CssValueParser<'a> {
    s: &'a str,
    pos: usize,
    length: usize,
}
impl<'a> CssValueParser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            s,
            pos: 0,
            length: s.chars().count()
        }
    }

    fn chars(&self) -> std::str::Chars<'a> {
        self.s[self.pos..].chars()
    }
    fn advance(&mut self, n: usize) {
        self.pos = self.length.min(self.pos + n);
    }
    fn char(&self) -> Option<char> {
        self.chars().next()
    }

    fn skip_spaces(&mut self) {
        let chars = self.chars().enumerate();
        for (n, c) in chars {
            if !c.is_whitespace() {
                self.pos += n;
                break;
            }
        }
    }
    fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.s[start..end]
    }

    fn read_until(&self, f: impl Fn(char) -> bool) -> &'a str {
        let start = self.pos;
        while let Some(char) = self.char() {
            if f(char) {
                break;
            }
        }

        self.slice(start, self.pos)
    }
}