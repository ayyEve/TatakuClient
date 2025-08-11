use crate::prelude::*;

pub(crate) struct OnlineContentManager {
    engines: Vec<Arc<dyn OnlineContentEngine>>,
    last_search: Option<Box<OnlineContentSearch>>,
}
impl OnlineContentManager {
    pub fn new(settings: &Settings) -> Self {
        Self {
            last_search: None,
            engines: vec![
                Arc::new(OsuDirect::new(settings)),
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
        action: OnlineContentAction,
        actions: &mut ActionQueue,
        values: &mut ValueCollection,
    ) {
        let data = &mut values
            .game
            .online_content
            .results;

        match action {
            OnlineContentAction::NextPage 
                => if let Some(last) = &mut self.last_search {
                    last.page += 1;
                    actions.push(OnlineContentAction::Search(last.clone()));
                },
            
            OnlineContentAction::PreviousPage 
                => if let Some(last) = &mut self.last_search {
                    if last.page == 0 { return }
                    last.page -= 1;
                    actions.push(OnlineContentAction::Search(last.clone()));
                },

            OnlineContentAction::SetPage(page) 
                => if let Some(last) = &mut self.last_search {
                    last.page = page as u32;
                    actions.push(OnlineContentAction::Search(last.clone()));
                },

            OnlineContentAction::Search(search) => {
                self.last_search = Some(search.clone());

                actions.push(OnlineContentSearchTask::new(
                    *search,
                    self.engines.clone(),
                ));
            }

            OnlineContentAction::Download(id) => {
                let index = data.items.iter()
                    .enumerate()
                    .find(|(_, i)| i.id == id)
                    .map(|(n, _)| n);

                let Some(index) = index else { return };
                let a = data.items.remove(index);
                actions.push(a.download);
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

                actions.push(SongAction::Set(SongSetAction::PushQueue));
                actions.push(AudioPreviewTask::new(preview.clone()));
            }
        }
    }
}
