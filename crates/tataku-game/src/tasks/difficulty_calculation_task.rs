use crate::prelude::*;
use tokio::sync::oneshot;

struct PendingCalc {
    mods: ModManager,
    abort: tokio::task::AbortHandle,
    receiver: oneshot::Receiver<(Box<dyn DiffCalc>, f32)>
}

/// A task to handle difficulty calculation for a single map
pub struct DiffCalcTask {
    state: TatakuTaskState,

    beatmap: Arc<BeatmapMeta>,
    info: GameModeInfo,

    diff_calc: Option<Box<dyn DiffCalc>>,
    failed_to_get_diff_calc: bool,

    diff_entries: Vec<(DifficultyEntry, f32)>,
    iter: DiffCalcTaskIter,

    inturrupted: Vec<ModManager>,
    current: Option<PendingCalc>,

    // abort_handle: Option<tokio::task::AbortHandle>
}
impl DiffCalcTask {
    pub fn new(
        beatmap: Arc<BeatmapMeta>, 
        info: GameModeInfo,
    ) -> Self {
        let mod_mutations = vec![HashSet::new()];

        Self {
            beatmap,
            info,
            state: TatakuTaskState::NotStarted,

            diff_calc: None,
            failed_to_get_diff_calc: false,

            diff_entries: Vec::new(),
            iter: DiffCalcTaskIter::new(mod_mutations),
            inturrupted: Vec::new(),

            // abort_handle: None,
            current: None
        }
    }

    async fn run_calc(
        &mut self,
        mods: ModManager,
        values: &mut dyn Reflect, 
    ) {
        let entry = DifficultyEntry::new(
            md5(self.info.id), 
            self.beatmap.beatmap_hash, 
            &mods
        );

        // try to load the diffcalc once
        if self.diff_calc.is_none() {
            // if we know we failed to get the diff calc, dont try again
            if self.failed_to_get_diff_calc {
                // debug!("calc failed");
                self.diff_entries.push((entry, -1.0)); // data.insert(diff_key, -1.0);
                return;
            }

            let settings = values.reflect_get("settings").unwrap();

            // otherwise, try to get the diff calc
            match self.info.create_diffcalc(&self.beatmap, &settings).await {
                Ok(c) => self.diff_calc = Some(c),
                Err(e) => {
                    error!("couldnt get calc: {e}");
                    self.failed_to_get_diff_calc = true;
                    self.diff_entries.push((entry, -1.0)); // data.insert(diff_key, -1.0);
                    return;
                }
            }
        }

        // now that we have a calc, run it
        let (sender, receiver) = oneshot::channel();

        let mods2 = mods.clone();
        let mut diff_calc = self.diff_calc.take().unwrap();
        let task = tokio::spawn(async move {
            // println!("diffcalcing!");
            let mut diff = 
                diff_calc
                .calc(&mods2)
                .await
                .unwrap_or_default()
                .diff;
            
            if !diff.is_normal() {
                diff = 0.0
            }

            #[cfg(feature="debug_perf_rating")]
            info!("[calc] {entry:?} -> {diff}");

            let _ = sender.send((diff_calc, diff));
        });

        self.current = Some(PendingCalc {
            abort: task.abort_handle(),
            mods,
            receiver
        });
    }

    async fn complete(&mut self, actions: &mut ActionQueue) {
        for (entry, diff) in self.diff_entries.take() {
            if let Err(e) = DifficultyManager::save_diff_entry(
                entry,
                diff
            ).await {
                actions.push(Notification::new_error("Failed to insert diff", e));
            }
        }

        self.state = TatakuTaskState::Complete
    }
}

#[async_trait]
impl TatakuTask for DiffCalcTask {
    fn get_id(&self) -> Cow<'static, str> { Cow::Borrowed("diff_calc") }
    fn get_name(&self) -> Cow<'static, str> { Cow::Owned(format!("Diff Calc for beatmap: {} and mode {}", self.beatmap.beatmap_hash, self.info.display_name)) }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(
        &mut self, 
        values: &mut dyn Reflect, 
        state: &TaskGameState, 
        actions: &mut ActionQueue
    ) {
        if state.ingame { 
            self.state = TatakuTaskState::Paused;

            // stop any existing calc
            if let Some(current) = self.current.take() {
                current.abort.abort();
                self.inturrupted.push(current.mods);
            }

            return;
        } else { 
            self.state = TatakuTaskState::Running;
        }

        // wait for the previous calc to complete
        if let Some(current) = &mut self.current {
            if let Ok((calc, diff)) = current.receiver.try_recv() {
                let entry = DifficultyEntry::new(
                    md5(self.info.id), 
                    self.beatmap.beatmap_hash, 
                    &current.mods
                );

                self.diff_calc = Some(calc);
                self.diff_entries.push((entry, diff));
                self.current = None;

                // if existing.contains_key(&diff_key) { continue }
                // self.diff_entries.push((diff_key, diff));
            }

            return
        }


        // try to get the next map
        if let Some(mods) = self.iter.next() {
            self.run_calc(mods, values).await;
        } 
        // try to get any inturrupted
        else if let Some(mods) = self.inturrupted.pop() {
            self.run_calc(mods, values).await;
        } else {
            // done
            self.complete(actions).await;
        }
    }
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
            Some(ModManager::new().with_mods(mods.iter()).with_speed(self.speed))
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
