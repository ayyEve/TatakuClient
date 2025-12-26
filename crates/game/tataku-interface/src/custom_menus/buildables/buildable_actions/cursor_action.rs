use crate::prelude::*;
use tataku::TatakuValue;
use common::reflect::Reflect;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BuildableCursorAction {
    Show,
    Hide,
}
#[cfg(feature="graphics")]
impl BuildableCursorAction {
    pub fn resolve(
        &self, 
        _values: &mut dyn Reflect, 
        _passed_in: Option<&TatakuValue>,
    ) -> Option<actions::cursor::CursorAction> {
        match self {
            Self::Show => Some(actions::cursor::CursorAction::SetVisible(true)),
            Self::Hide => Some(actions::cursor::CursorAction::SetVisible(false)),
        }
    }
    
    pub fn build(&mut self) {}
}
