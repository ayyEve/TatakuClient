use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub enum BuildableChatAction {
    SendMessage {
        channel: BuildableTextTag,
        message: BuildableTextTag,
    },

    OpenChannel {
        #[serde(rename="$value", default)]
        channel: Option<BuildableText>,
        #[serde(rename="channel", default)]
        channel_tag: Option<BuildableTextTag>,

        #[serde(default)]
        password: Option<BuildableTextTag>,
    },

    CloseChannel {
        #[serde(rename="$value", default)]
        channel: Option<BuildableText>,
        #[serde(rename="channel", default)]
        channel_tag: Option<BuildableTextTag>,
    }
}
impl BuildableChatAction {
    pub fn into_action(
        self, 
        values: &dyn Reflect, 
        _passed_in: &Option<TatakuValue>
    ) -> Option<TatakuAction> {
        match self {
            Self::SendMessage { 
                channel, 
                message 
            } => Some(ChatAction::SendMessage { 
                channel: channel.to_string(values), 
                message: message.to_string(values),
            }.into()),

            Self::OpenChannel { 
                channel, 
                channel_tag ,
                password
            } => Some(ChatAction::OpenChannel { 
                channel: channel.or(channel_tag.map(|i| i.value))?.to_string(values),
                password: password.map(|i| i.to_string(values)),
            }.into()),

            Self::CloseChannel { 
                channel, 
                channel_tag 
            } => Some(ChatAction::CloseChannel { 
                channel: channel.or(channel_tag.map(|i| i.value))?.to_string(values),
            }.into()),
        }
    }

    pub fn build(&mut self, _values: &dyn Reflect) {
        match self {
            Self::SendMessage { 
                channel,
                message
            } => {
                let _ = channel.compute();
                let _ = message.compute();
            }

            Self::OpenChannel { 
                channel, 
                channel_tag ,
                password,
            } => {
                if let Some(channel) = channel {
                    let _ = channel.compute();
                }
                if let Some(channel) = channel_tag {
                    let _ = channel.compute();
                }
                if let Some(password) = password {
                    let _ = password.compute();
                }
            }

            Self::CloseChannel { channel, channel_tag } => {
                if let Some(channel) = channel {
                    let _ = channel.compute();
                }
                if let Some(channel) = channel_tag {
                    let _ = channel.compute();
                }
            }
        };
    }
}
