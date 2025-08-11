
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
