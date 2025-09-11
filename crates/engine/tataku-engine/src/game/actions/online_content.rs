use crate::*;
use engine::online_content::OnlineContentSearch;

#[derive(Clone, Debug)]
pub enum OnlineContentAction {
    Search(Box<OnlineContentSearch>),
    Download(usize),
    AudioPreview(usize),

    NextPage,
    PreviousPage,
    SetPage(usize),
}
impl From<OnlineContentAction> for actions::Action {
    fn from(value: OnlineContentAction) -> Self {
        Self::OnlineContent(value)
    }
}
