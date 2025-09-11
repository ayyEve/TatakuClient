use crate::*;

pub trait TatakuTask: Send + Sync {
    fn get_id(&self) -> CowStr { self.get_name() }
    fn get_name(&self) -> CowStr;
    fn get_type(&self) -> TatakuTaskType;
    fn get_state(&self) -> TatakuTaskState;

    fn run(
        &mut self, 
        values: &mut dyn common::reflect::Reflect, 
        state: &TaskGameState, 
        actions: &mut actions::ActionQueue
    ); 
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
    #[default] NotStarted,

    /// This task is currently running
    Running,

    /// This task is currently paused
    Paused,

    /// This task has been completed
    Complete,
}
