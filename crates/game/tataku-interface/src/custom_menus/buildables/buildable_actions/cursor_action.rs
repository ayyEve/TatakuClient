use crate::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildableCursorAction {
    Show,
    Hide,
}
impl BuildableCursorAction {
    pub fn into_action(
        self, 
        _values: &mut dyn Reflect, 
        _passed_in: Option<&TatakuValue>
    ) -> Option<CursorAction> {
        match self {
            Self::Show => Some(CursorAction::SetVisible(true)),
            Self::Hide => Some(CursorAction::SetVisible(false)),
        }
    }
    
    pub fn build(&mut self, _values: &dyn Reflect) {}
}
