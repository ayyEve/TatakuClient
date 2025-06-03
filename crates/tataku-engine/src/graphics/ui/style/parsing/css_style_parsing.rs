use taffy::*;
use crate::prelude::*;
use crate::prelude::ui::*;
use super::value_parser::*;


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


// parsing
#[allow(clippy::result_unit_err, reason = "we dont care about the error")] 
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
        if s.starts_with("rgb") {
            let mut parser = CssValueParser::new(s);
            parser.read_until(|c| c == '(');
            parser.advance(1);
            parser.skip_spaces();

            let r = parser.read_until(|c| c == ',').trim();
            parser.advance(1);
            parser.skip_spaces();
            let g = parser.read_until(|c| c == ',').trim();
            parser.advance(1);
            parser.skip_spaces();
            let b = parser.read_until(|c| c == ',' || c == ')').trim();
            let mut a = "255";
            if parser.char() == Some(',') {
                parser.advance(1);
                parser.skip_spaces();
                a = parser.read_until(|c| c == ')');
            }

            let r = r.parse::<u8>().map_err(|_| ())?;
            let g = g.parse::<u8>().map_err(|_| ())?;
            let b = b.parse::<u8>().map_err(|_| ())?;
            let a = a.parse::<u8>().map_err(|_| ())?;
            Ok(Color::from_rgba8(r, g, b, a))
        } else {
            Color::try_from_hex(s).ok_or(())
        }
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
        parse_image_fit, ImageStretch, 
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
