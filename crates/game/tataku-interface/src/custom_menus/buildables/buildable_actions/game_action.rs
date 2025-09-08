use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableGameAction {
    /// Quit the game
    Quit,

    /// Refresh Scores
    RefreshScores,

    /// View a score by id
    ViewScore {
        #[serde(rename="$value")]
        score: BuildableValue
    },

    #[serde(rename="notification")]
    ShowNotification {
        #[serde(alias = "@color")] color: Color,
        // Attributes need to be wrapped so that they can be parsed as strings.
        #[serde(rename = "@duration", default)] duration_attribute: Option<TatakuValue>,
        #[serde(default)] duration: Option<BuildableValue>,
        #[serde(rename = "$value")]
        text: Vec<BuildableText>,
    },

    CopyToClipboard {
        #[serde(rename="$value")]
        text: Vec<BuildableText>,
    },
}

impl BuildableGameAction {
    pub fn resolve(
        &self,
        values: &mut dyn Reflect,
        passed_in: Option<&TatakuValue>
    ) -> Option<GameAction> {
        match self {
            Self::CopyToClipboard { text } => {
                let text: String = text.iter()
                    .map(|text| text.to_string(values))
                    .collect();

                Some(GameAction::CopyToClipboard(text.into()))
            }

            Self::Quit => Some(GameAction::Quit),
            Self::RefreshScores => Some(GameAction::RefreshScores),
            Self::ShowNotification {
                text,
                color,
                duration_attribute,
                duration,
            } => {
                let duration = duration_attribute
                    .clone()
                    .map(BuildableValue::Value)
                    .or(duration.clone())?;

                let text: String = text.iter()
                    .map(|text| text.to_string(values))
                    .collect();

                Some(GameAction::AddNotification(Notification::new(
                    text,
                    *color,
                    duration.resolve(values, passed_in)?.as_f32()?,
                    NotificationOnClick::None
                )))
            },

            Self::ViewScore { score } => {
                let score_id = score
                    .resolve(values, passed_in)?
                    .as_u64()
                    ? as usize;

                debug!("score: {score_id}");

                Some(GameAction::ViewScoreId(score_id))
            }
        }
    }

    pub fn build(&mut self) {
        match self {
            Self::ViewScore {
                score
            } => score.build(),
            Self::ShowNotification {
                text,
                duration,
                ..
            } => {
                for i in text {
                    if let Err(e) = i.compute() {
                        error!("error parsing text '{i:?}': {e:?}");
                    }
                }

                if let Some(duration) = duration {
                    duration.build();
                }
            }

            Self::Quit => {},
            Self::RefreshScores => {},
            Self::CopyToClipboard { .. } => {},
        };
    }
}
