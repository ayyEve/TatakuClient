use super::super::parsing::value_parser::CssValueParser;

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
