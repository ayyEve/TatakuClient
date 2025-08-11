
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
