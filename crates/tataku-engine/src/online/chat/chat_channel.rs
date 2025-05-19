use crate::prelude::*;

// some kind of identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Reflect)]
#[reflect(display="display")]
pub enum ChatChannelType {
    Channel { 
        name: String 
    },
    User {
        username: String
    }
}
impl ChatChannelType {
    pub fn from_name(name: String) -> ChatChannelType {
        if name.starts_with("#") {
            ChatChannelType::Channel { name: name.trim_start_matches("#").to_string() }
        } else {
            ChatChannelType::User { username: name }
        }
    }
    pub fn get_name(&self) -> Cow<'_, str> {
        match self {
            ChatChannelType::Channel { name } => Cow::Owned(format!("#{name}")),
            ChatChannelType::User { username } => Cow::Borrowed(username),
        }
    }
}
impl std::str::FromStr for ChatChannelType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_name(s.to_owned()))
    }
}
impl std::fmt::Display for ChatChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_name().fmt(f)
    }
}

impl std::cmp::PartialEq<String> for ChatChannelType {
    fn eq(&self, other: &String) -> bool {
        match self {
            Self::Channel { name } => name == other,
            Self::User { username } => username == other,
        }
    }
}


#[derive(Debug, Clone)]
#[derive(Reflect)]
#[reflect(display="debug")]
pub struct ChatChannel {
    #[reflect(alias("name"))]
    pub channel_type: ChatChannelType,
    pub messages: Vec<ChatMessage>,
}
