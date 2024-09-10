use crate::prelude::*;



#[async_trait]
pub trait TatakuTask: Send + Sync  {
    fn get_name(&self) -> Cow<'static, str>;
    fn get_type(&self) -> TatakuTaskType;
    fn get_state(&self) -> TatakuTaskState;

    async fn run(&mut self, values: &mut dyn Reflect, state: &TaskGameState, actions: &mut ActionQueue); 
}

pub struct TaskGameState {
    /// Current game time in ms
    pub game_time: u64,

    /// Are we currently in a game?
    pub ingame: bool,
}

/// What kind of task is the task?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TatakuTaskType {
    /// This task runs continuously
    Continuous,

    /// This task runs once
    Once,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TatakuTaskState {
    /// This task hasn't started yet
    #[default]
    NotStarted,

    /// This task is currently running
    Running,

    /// This task is currently paused
    Paused,

    /// This task has been completed
    Complete,
}
