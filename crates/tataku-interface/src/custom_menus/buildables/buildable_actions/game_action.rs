use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildableGameAction {
    /// Quit the game
    Quit,

    /// Refresh Scores
    RefreshScores,

    /// View a score by id
    ViewScore { 
        #[serde(rename="$value", alias="$text")] 
        score: BuildableValue
    },
    
    #[serde(rename="notification")]
    ShowNotification {
        text: BuildableTextTag,
        #[serde(alias = "@color")] color: Color,
        duration: BuildableValueTag,
    },

    CopyToClipboard {
        #[serde(alias="$value")]
        text: BuildableText,
    },
}
impl BuildableGameAction {
    pub fn into_action(
        self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<GameAction> {
        match self {
            Self::CopyToClipboard { mut text } => {
                let _ = text.compute();
                let text = text.to_string(values);
                Some(GameAction::CopyToClipboard(text))
            }

            Self::Quit => Some(GameAction::Quit),
            Self::RefreshScores => Some(GameAction::RefreshScores),
            Self::ShowNotification {
                text, 
                color, 
                duration
            } => Some(GameAction::AddNotification(Notification::new(
                text.to_string(values),
                color,
                duration.resolve(values, passed_in)?.as_f32().ok()?,
                NotificationOnClick::None
            ))),

            Self::ViewScore { score } => {
                let score_id = score
                    .resolve(values, passed_in)?
                    .as_u64()
                    .ok()? as usize;

                debug!("score: {score_id}");

                Some(GameAction::ViewScoreId(score_id))
            }
        }
    }
    
    pub fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::ViewScore { 
                score 
            } => score.resolve_pre(values),
            Self::ShowNotification { 
                text, 
                duration, 
                .. 
            } => {
                if let Err(e) = text.compute() {
                    error!("error parsing text '{text:?}': {e:?}");
                }

                duration.resolve_pre(values);
            }
            
            Self::Quit => {},
            Self::RefreshScores => {},
            Self::CopyToClipboard { .. } => {},
        };
    }
}
