use crate::*;
use core::result::Result;
use super::value_parser::*;
use graphics::ImageStretch;
use graphics::TextureSource;


macro_rules! impl_parse {
    ($fn: ident, $struct: ident, $(($i: expr, $v: tt));*) => {
        pub(crate) fn $fn(s: &str) -> Result<$struct, ()> {
            match s {
                $( $i => Ok($struct::$v), )*
                _ => Err(())
            }
        }
    };

    (rect, $fn: ident, $parent_fn: ident :: $parent_fn2: ident, $struct: ident) => {
        pub(crate) fn $fn(s: &str) -> Result<Rect<$struct>, ()> {
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
impl crate::style::css::CssStyle {
    pub(crate) fn parse_color(s: &str) -> Result<Color, ()> {
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
            Ok(Color::new_rgba8(r, g, b, a))
        } else {
            Color::try_from_hex(s).ok_or(())
        }
    }

    pub(crate) fn parse_image_source(s: &str) -> Result<TextureSource, ()> {
        match s {
            "raw" => Ok(TextureSource::Raw),
            "skin" => Ok(TextureSource::Skin),
            "default-skin" => Ok(TextureSource::DefaultSkin),
            other => Ok(TextureSource::Beatmap(other.to_owned()))
        }
    }


    impl_parse!(
        parse_image_fit, ImageStretch,
        ("fill", Fill);
        ("none", None);
        ("cover", Cover);
        ("contain", Contain)
    );

    impl_parse!(
        parse_font, DefaultFont,
        ("main", Main);
        ("font-awesome", FontAwesome);
        ("icon", FontAwesome);
        ("icons", FontAwesome);
        ("fallback", Fallback)
    );

}
