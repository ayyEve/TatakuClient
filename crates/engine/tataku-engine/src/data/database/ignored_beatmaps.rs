use crate::*;
use common::Md5Hash;

#[derive(Clone, Debug, Default2)]
#[derive(Serialize, Deserialize)]
pub enum IgnoredBeatmap {
    #[default]
    Path(String),
    Hash(Md5Hash),
}
impl IgnoredBeatmap {
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Self::Path(s) => Cow::Borrowed(s.as_str()),
            Self::Hash(h) => Cow::Owned(h.to_string()),
        }
    }
}
