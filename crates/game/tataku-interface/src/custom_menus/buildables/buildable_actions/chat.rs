use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableChatAction {
    SendMessage {
        channel: Wrapped<BuildableValue>,
        #[serde(rename="$value")]
        message: Vec<BuildableText>,
    },

    OpenChannel {
        #[serde(alias="$value", default)]
        channel: BuildableValue,

        #[serde(default)]
        password: Option<Wrapped<BuildableValue>>,
    },

    CloseChannel {
        #[serde(alias="$value", default)]
        channel: BuildableValue
    }
}
impl BuildableChatAction {
    pub fn into_action(
        self, 
        values: &dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<TatakuAction> {
        match self {
            Self::SendMessage { 
                channel, 
                message 
            } => {
                let message: String = message.into_iter()
                    .map(|mut text| {
                        let _ = text.compute();
                        text.to_string(values)
                    }).collect();

                Some(ChatAction::SendMessage {
                    channel: channel.inner.resolve(values, passed_in).unwrap().as_string(),
                    message,
                }.into())
            },

            Self::OpenChannel { 
                channel, 
                password
            } => Some(ChatAction::OpenChannel { 
                channel: channel.resolve(values, passed_in).unwrap().as_string(),
                password: password
                    .and_then(|i| i.inner.resolve(values, passed_in).map(|i| i.as_string())),
            }.into()),

            Self::CloseChannel { 
                channel, 
            } => Some(ChatAction::CloseChannel { 
                channel: channel.resolve(values, passed_in).unwrap().as_string(),
            }.into()),
        }
    }

    pub fn build(&mut self, _values: &dyn Reflect) {
        match self {
            Self::SendMessage { 
                channel,
                message
            } => {
                // let _ = channel.compute();
                // let _ = message.compute();
            }

            Self::OpenChannel { 
                channel, 
                password,
            } => {
                // let _ = channel.compute();

                // if let Some(password) = password {
                //     let _ = password.compute();
                // }
            }

            Self::CloseChannel { channel } => {
                // let _ = channel.compute();
            }
        };
    }
}
