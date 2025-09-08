use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BuildableCursorAction {
    Show,
    Hide,
}
impl BuildableCursorAction {
    pub fn resolve(
        &self, 
        _values: &mut dyn Reflect, 
        _passed_in: Option<&TatakuValue>,
    ) -> Option<CursorAction> {
        match self {
            Self::Show => Some(CursorAction::SetVisible(true)),
            Self::Hide => Some(CursorAction::SetVisible(false)),
        }
    }
    
    pub fn build(&mut self) {}
}
