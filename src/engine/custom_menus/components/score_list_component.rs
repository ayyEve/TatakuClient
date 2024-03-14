use crate::prelude::*;

pub struct ScoreListComponent {
    actions: ActionQueue,

    /// what we think the playmode is currently
    mode: SYValueHelper,

    /// what we think the current map hash is
    /// string because thats how its stored in the variable collection
    map_hash: SYValueHelper,

    /// what method to use to retreive scores
    method: Option<SYValueHelper>,

    score_helper: ScoreHelper,

    current_scores: Vec<IngameScore>,
    score_loader: Option<Arc<AsyncRwLock<ScoreLoaderHelper>>>,
}
impl ScoreListComponent {
    pub fn new(method_var: Option<String>) -> Self {
        let score_helper = ScoreHelper::new();

        Self {
            actions: ActionQueue::new(),
            mode: SYValueHelper::new("global.playmode", String::new()),
            map_hash: SYValueHelper::new("map.hash", String::new()),
            method: method_var.map(|var| SYValueHelper::new(var, format!("{:?}", score_helper.current_method))),
            score_helper,

            current_scores: Vec::new(),
            score_loader: None,
        }
    }
}

impl ScoreListComponent {
    pub async fn load_scores(&mut self, values: &mut ShuntingYardValues) {
        trace!("Score reload requested");

        // clear scores, and make sure the values in the collection are removed as well
        self.current_scores.clear();
        self.update_values(values);

        // get the map hash and mode, and make sure they're actually set
        let map_hash = self.map_hash.as_string();
        let mode = self.mode.as_string();
        if map_hash.is_empty() || mode.is_empty() { return }

        if let Some(Ok(method)) = self.method.as_deref().map(TryInto::try_into) {
            self.score_helper.current_method = method;
        }

        let Ok(hash) = Md5Hash::try_from(&map_hash) else { return };
        let Some(map) = BEATMAP_MANAGER.read().await.get_by_hash(&hash) else { return };

        trace!("Reloading scores");
        self.score_loader = Some(self.score_helper.get_scores(hash, &map.check_mode_override(mode)).await);
    }


    fn update_values(&self, values: &mut ShuntingYardValues) {
        let list = self.current_scores.iter().enumerate().map(|(n, score)| {
            let score:CustomElementValue = score.into();
            let mut data = score.as_map_helper().unwrap();
            data.set("id", n as u64);

            data.finish()
        }).collect::<Vec<_>>();

        values.set("score_list.scores", list);
    }
}

#[async_trait]
impl Widgetable for ScoreListComponent {
    async fn update(&mut self, values: &mut ShuntingYardValues, _actions: &mut ActionQueue) {

        // check if map hash or playmode have changed
        if self.mode.check(values) {
            trace!("mode changed");
            self.load_scores(values).await;
        }
        if self.map_hash.check(values) {
            trace!("hash changed");
            self.load_scores(values).await;
        }
        if let Some(method) = &mut self.method {
            if method.check(values) {
                trace!("method changed");
                self.load_scores(values).await;
            }
        }


        // check load score
        if let Some(helper) = self.score_loader.clone() {
            let helper = helper.read().await;

            if helper.done {
                self.score_loader = None;

                // load scores
                self.current_scores = helper.scores.clone();
                self.current_scores.sort_by(|a, b| b.score.score.cmp(&a.score.score));
                trace!("Got list of {} scores from {:?}", self.current_scores.len(), self.score_helper.current_method);

                self.update_values(values);
            }
        }
    }


    async fn handle_message(&mut self, message: &Message, _values: &mut ShuntingYardValues) -> Vec<MenuAction> { 
        if let MessageTag::String(tag) = &message.tag {
            match &**tag {
                "score_list.open_score" => {
                    let id = match &message.message_type {
                        MessageType::Number(n) => *n,
                        MessageType::Value(CustomElementValue::U64(n)) => (*n) as usize,
                        other => {
                            error!("invalid type for score id: {other:?}");
                            return Vec::new();
                        }
                    };

                    if let Some(score) = self.current_scores.get(id).cloned() {
                        self.actions.push(GameMenuAction::ViewScore(score));
                    } else { warn!("score not found") }

                }

                _ => {}
            }
        }


        self.actions.take()
    }
}