use crate::prelude::*;
use engine::{
    game::task::*,
    io::AsyncLoader,
    online_content::*,
};

pub struct OnlineContentSearchTask {
    engines: Vec<Arc<dyn OnlineContentEngine>>,
    loader: Option<AsyncLoader<OnlineContentSearchResults>>,
    search: OnlineContentSearch,

    state: TatakuTaskState,
}
impl OnlineContentSearchTask {
    pub fn new(
        search: OnlineContentSearch,
        engines: Vec<Arc<dyn OnlineContentEngine>>,
    ) -> Self {
        Self {
            search,
            loader: None,
            engines,
            state: TatakuTaskState::NotStarted,
        }
    }

    fn results(values: &mut ValueCollection) -> &mut OnlineContentReflectResults {
        &mut values.game.online_content.results
    }
}
impl TatakuTask for OnlineContentSearchTask {
    fn get_name(&self) -> CowStr { "OnlineContentSearch".into() }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(&mut self, shell: &mut TaskShell) {
        let values = ValueCollection::from_reflect_mut(shell.values);

        if self.loader.is_none() {
            let results = Self::results(values);
            results.completed = false;
            results.items.clear();

            let engine = self
                .engines
                .iter_mut()
                .find(|i | 
                    i.capabilities().engine_id == self.search.engine_id)
                ;

            let Some(engine) = engine 
            else {
                error!("Missing search engine?? {}", self.search.engine_id);
                results.completed = true;
                self.state = TatakuTaskState::Complete;
                return;
            };

            self.loader = Some(engine.search(&values.settings, self.search.clone()));
            
            self.state = TatakuTaskState::Running;
            return;
        }

        let Some(loader) = &self.loader 
        else { 
            Self::results(values).completed = true;
            self.state = TatakuTaskState::Complete;
            return;
        };

        let Some(mut search_results) = loader.check() 
        else { return };

        // remove maps from the list that we already have
        let beatmaps = &values.beatmap_manager.beatmaps;

        search_results.items.retain(|i| {
            #[allow(irrefutable_let_patterns, reason = "expandability")]
            if let OnlineContentItemType::Map { 
                map_hashes, 
                .. 
            } = &i.item_type {
                !map_hashes
                    .iter()
                    .any(|i| beatmaps.contains_key(i))
            } else {
                true
            }
        });

        let results = Self::results(values);
        results.items = search_results.items;
        results.count = search_results.count;
        results.page = self.search.page as usize;
        results.completed = true;

        self.state = TatakuTaskState::Complete;
    }
}
