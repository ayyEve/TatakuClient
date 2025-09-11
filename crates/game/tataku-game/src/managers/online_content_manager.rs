use crate::prelude::*;

use engine::{
    actions,
    online_content::*,
};

pub(crate) struct OnlineContentManager {
    engines: Vec<Arc<dyn OnlineContentEngine>>,
    last_search: Option<Box<OnlineContentSearch>>,
}
impl OnlineContentManager {
    pub fn new(settings: &engine::Settings) -> Self {
        Self {
            last_search: None,
            engines: vec![
                Arc::new(engine::online_content::OsuDirect::new(settings)),
            ],
        }
    }

    pub fn get_capabilities(&self) -> Vec<OnlineContentCapabilities> {
        self.engines
            .iter()
            .map(|i| i.capabilities())
            .cloned()
            .collect()
    }

    pub fn handle_action(
        &mut self, 
        action: actions::online_content::OnlineContentAction,
        actions: &mut actions::ActionQueue,
        values: &mut ValueCollection,
    ) {
        use actions::online_content::OnlineContentAction as OnlineContentAction;
        let data = &mut values
            .game
            .online_content
            .results;

        match action {
            OnlineContentAction::NextPage 
                => if let Some(last) = &mut self.last_search {
                    last.page += 1;
                    actions.push(OnlineContentAction::Search(last.clone()).into());
                },
            
            OnlineContentAction::PreviousPage 
                => if let Some(last) = &mut self.last_search {
                    if last.page == 0 { return }
                    last.page -= 1;
                    actions.push(OnlineContentAction::Search(last.clone()).into());
                },

            OnlineContentAction::SetPage(page) 
                => if let Some(last) = &mut self.last_search {
                    last.page = page as u32;
                    actions.push(OnlineContentAction::Search(last.clone()).into());
                },

            OnlineContentAction::Search(search) => {
                self.last_search = Some(search.clone());

                actions.push(OnlineContentSearchTask::new(
                    *search,
                    self.engines.clone(),
                ).into());
            }

            OnlineContentAction::Download(id) => {
                let index = data.items.iter()
                    .enumerate()
                    .find(|(_, i)| i.id == id)
                    .map(|(n, _)| n);

                let Some(index) = index else { return };
                let a = data.items.remove(index);
                actions.push(a.download.into());
            }

            OnlineContentAction::AudioPreview(id) => {
                let index = data.items.iter()
                    .enumerate()
                    .find(|(_, i)| i.id == id)
                    .map(|(n, _)| n);

                let Some(index) = index else { return };

                let Some(a) = data.items.get(index) 
                else { return };

                let Some(preview) = &a.audio_preview 
                else { return };

                actions.push(actions::song::SongAction::Set(
                    actions::song::SongSetAction::PushQueue
                ).into());
                actions.push(AudioPreviewTask::new(preview.clone()).into());
            }
        }
    }
}
