use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildableGameAction {
    /// Quit the game
    Quit,

    /// View a score by id
    ViewScore { 
        #[serde(rename="$value", alias="$text")] 
        score: BuildableValueTag
    },
    
    #[serde(rename="notification")]
    ShowNotification {
        text: BuildableTextTag,
        #[serde(alias = "@color")] color: Color,
        duration: BuildableValueTag
    },
}
impl BuildableGameAction {
    pub fn into_action(self, values: &mut dyn Reflect, passed_in: Option<TatakuValue>) -> Option<GameAction> {
        match self {
            Self::Quit => Some(GameAction::Quit),
            Self::ShowNotification {
                text, color, duration
            } => Some(GameAction::AddNotification(Notification::new(
                text.to_string(values),
                color,
                duration.resolve(values, passed_in)?.as_f32().ok()?,
                NotificationOnClick::None
            ))),

            Self::ViewScore { score } => {
                Some(GameAction::ViewScoreId(score.resolve(values, passed_in)?.as_u64().ok()? as usize))
            }
        }
    }
    
    pub fn build(&mut self, values: &dyn Reflect) {
        let thing = match self {
            Self::ViewScore { score } => score,
            Self::ShowNotification { text, duration, .. } => {
                if let Err(e) = text.compute() {
                    error!("error parsing text '{text:?}': {e:?}");
                }

                duration
            }
            Self::Quit => return,
        };

        thing.value.resolve_pre(values);
    }
}
