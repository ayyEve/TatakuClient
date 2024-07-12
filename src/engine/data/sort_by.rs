use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    #[default]
    Title,
    Artist,
    Creator,
    Difficulty,
}
impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl TryFrom<&TatakuValue> for SortBy {
    type Error = String;
    fn try_from(value: &TatakuValue) -> Result<Self, Self::Error> {
        match value {
            TatakuValue::String(s) => {
                match &**s {
                    "Title" | "title" => Ok(Self::Title),
                    "Artist" | "artist" => Ok(Self::Artist),
                    "Creator" | "creator" => Ok(Self::Creator),
                    "Difficulty" | "difficulty" => Ok(Self::Difficulty),
                    other => Err(format!("invalid SortBy str: '{other}'"))
                }
            }
            TatakuValue::U64(n) => {
                match *n {
                    0 => Ok(Self::Title),
                    1 => Ok(Self::Artist),
                    2 => Ok(Self::Creator),
                    3 => Ok(Self::Difficulty),
                    other => Err(format!("Invalid SortBy number: {other}")),
                }
            }

            other => Err(format!("Invalid SortBy value: {other:?}"))
        }
    }
}

impl Into<TatakuValue> for SortBy {
    fn into(self) -> TatakuValue {
        TatakuValue::String(self.to_string())
    }
}

impl SortBy {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Title,
            Self::Artist,
            Self::Creator,
            Self::Difficulty
        ]
    }

    pub fn to_string(&self) -> String {
        format!("{self:?}")
    }

    pub fn from_str(s: &String) -> Option<Self> {
        for i in Self::list() {
            if s == &format!("{i:?}") { return Some(i) };
        }
        None
    }
}
