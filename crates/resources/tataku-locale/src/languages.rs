

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum Language {
    #[default]
    English,
}
impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
        }
    }

    pub fn from_code(code: impl AsRef<str>) -> Option<Self> {
        let code = code.as_ref();

        match code {
            "en" => Some(Self::English),
            _ => None,
        }
    }
}