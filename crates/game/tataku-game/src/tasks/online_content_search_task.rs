use crate::prelude::*;

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

    fn results(values: &mut dyn Reflect) -> &mut OnlineContentReflectResults {
        values
            .reflect_get_mut::<OnlineContentReflectResults>(
                "game.online_content.results"
            )
            .unwrap()
    }
}
impl TatakuTask for OnlineContentSearchTask {
    fn get_name(&self) -> CowStr { "OnlineContentSearch".into() }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(
        &mut self, 
        values: &mut dyn Reflect, 
        _state: &TaskGameState, 
        _actions: &mut ActionQueue,
    ) {
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

            let settings = values
                .reflect_get::<Settings>("settings")
                .unwrap();
            self.loader = Some(engine.search(&settings, self.search.clone()));
            
            self.state = TatakuTaskState::Running;
            return;
        }

        let Some(loader) = &self.loader 
        else { 
            let results = Self::results(values);

            results.completed = true;
            self.state = TatakuTaskState::Complete;
            return;
        };

        let Some(mut search_results) = loader.check() 
        else { return };

        // remove maps from the list that we already have
        let beatmaps = values
            .reflect_get::<HashMap<Md5Hash, Arc<BeatmapMeta>>>(
                "beatmaps.beatmaps_by_hash"
            )
            .unwrap();

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
