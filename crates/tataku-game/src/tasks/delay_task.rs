use crate::prelude::*;

pub struct DelayTask {
    state: TatakuTaskState,

    /// The task to run once the delay has elapsed
    task: Option<Box<dyn TatakuTask>>,

    /// How many ms the task should be delayed
    delay: u64,

    /// When was the task started?
    start_time: u64,
}
impl DelayTask {
    pub fn new(task: impl TatakuTask + 'static, delay: u64) -> Self {
        Self::new_from_boxed(Box::new(task), delay)
    }

    pub fn new_from_boxed(task: Box<dyn TatakuTask>, delay: u64) -> Self {
        Self {
            state: TatakuTaskState::NotStarted,
            task: Some(task),
            delay,
            start_time: 0
        }
    }
}

#[async_trait]
impl TatakuTask for DelayTask {
    fn get_name(&self) -> Cow<'static, str> {
        if let Some(task) = &self.task {
            Cow::Owned(format!("Delayed ({})", task.get_name()))
        } else {
            Cow::Borrowed("Delayed (Fulfilled)")
        }
    }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(
        &mut self,
        _values: &mut dyn Reflect, 
        state: &TaskGameState, 
        actions: &mut ActionQueue
    ) {
        if matches!(self.state, TatakuTaskState::NotStarted) {
            self.start_time = state.game_time;
            self.state = TatakuTaskState::Running;
            return;
        }

        if (state.game_time - self.start_time) >= self.delay {
            self.state = TatakuTaskState::Complete;
            if let Some(task) = self.task.take() {
                actions.push(TaskAction::AddTask(task));
            }
        }
    }
}