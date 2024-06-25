use crate::prelude::*;

/// A task to handle difficulty calculation for a single map
pub struct DiffCalcTask {
    state: TatakuTaskState,

    beatmap: Arc<BeatmapMeta>,
    playmode: String,

    diff_calc: Option<Box<dyn DiffCalc>>,
    failed_to_get_diff_calc: bool,

    diff_entries: Vec<(DifficultyEntry, f32)>,

    iter: DiffCalcTaskIter,
}
impl DiffCalcTask {
    pub fn new(beatmap: Arc<BeatmapMeta>, playmode: String) -> Self {
        let mod_mutations = vec![HashSet::new()];

        Self {
            beatmap,
            playmode,
            state: TatakuTaskState::NotStarted,

            diff_calc: None,
            failed_to_get_diff_calc: false,

            diff_entries: Vec::new(),

            iter: DiffCalcTaskIter::new(mod_mutations)
        }
    }
}

#[async_trait]
impl TatakuTask for DiffCalcTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Owned(format!("Diff Calc for beatmap: {} and mode {}", self.beatmap.beatmap_hash, self.playmode)) }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(&mut self, _values: &mut ValueCollection, state: &TaskGameState, _actions: &mut ActionQueue) {
        if state.ingame { 
            self.state = TatakuTaskState::Paused;
            return;
        } else { 
            self.state = TatakuTaskState::Running 
        }

        // TODO: move this to another thread ?
        if let Some(mods) = self.iter.next() {
            let diff_key = DifficultyEntry::new(self.beatmap.beatmap_hash, &mods);
            // if existing.contains_key(&diff_key) { continue }

            // try to load the calc once
            if self.diff_calc.is_none() {

                // if we know we failed to get the diff calc, dont try again
                if self.failed_to_get_diff_calc {
                    // debug!("calc failed");
                    self.diff_entries.push((diff_key, -1.0)); // data.insert(diff_key, -1.0);
                    return;
                }

                // otherwise, try to get the diff calc
                match calc_diff(&self.beatmap, self.playmode.clone()).await {
                    Ok(c) => self.diff_calc = Some(c),
                    Err(e) => {
                        error!("couldnt get calc: {e}");
                        self.failed_to_get_diff_calc = true;
                        self.diff_entries.push((diff_key, -1.0)); // data.insert(diff_key, -1.0);
                        return;
                    }
                }
            }
            
            let diff = self.diff_calc.as_mut().unwrap().calc(&mods).await.unwrap_or_default().diff.normal_or(0.0);
            
            #[cfg(feature="debug_perf_rating")]
            info!("[calc] {diff_key:?} -> {diff}");
            self.diff_entries.push((diff_key, diff));
        } else {
            self.state = TatakuTaskState::Complete
        }

    }
}



struct DiffCalcTaskInner {
    playmode: String,
    beatmap: Arc<BeatmapMeta>,
}


struct DiffCalcTaskIter {
    speed: u16,
    mod_mutations: Vec<HashSet<String>>,

    speed_iter: Box<dyn Iterator<Item = u16> + Send + Sync>,
    mods_iter: Box<dyn Iterator<Item = HashSet<String>> + Send + Sync>
}
impl DiffCalcTaskIter {
    pub fn new(mod_mutations: Vec<HashSet<String>>) -> Self {
        let mut speed_iter = Box::new((50..=1000).step_by(5));
        let speed = speed_iter.next().unwrap();
        let mods_iter = Box::new(mod_mutations.clone().into_iter());

        Self {
            speed,
            mod_mutations,

            speed_iter,
            mods_iter,
        }
    }
}

impl Iterator for DiffCalcTaskIter {
    type Item = ModManager;
    fn next(&mut self) -> Option<Self::Item> {
        // get the next set of mods
        if let Some(mods) = self.mods_iter.next() {
            Some(ModManager::new().with_mods(mods).with_speed(self.speed))
        } else if let Some(speed) = self.speed_iter.next() {
            // otherwise, get the next speed, and reset the mods iter
            self.speed = speed;
            self.mods_iter = Box::new(self.mod_mutations.clone().into_iter());
            self.next()
        } else {
            // otherwise, we're done
            None
        }
    }
}