use crate::prelude::*;


/// just runs the action, primarily used with the DelayedTask to run actions after a certain amount of time
pub struct ActionTask {
    state: TatakuTaskState,
    action: Option<TatakuAction>,
}
impl ActionTask {
    pub fn new(action: impl Into<TatakuAction>) -> Self {
        Self {
            action: Some(action.into()),
            state: TatakuTaskState::NotStarted,
        }
    }
}

// overengineered lol
#[async_trait]
impl TatakuTask for ActionTask {
    fn get_name(&self) -> Cow<'static, str> { "action_task".into() }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }
    
    async fn run(&mut self, _values: &mut dyn Reflect, _state: &TaskGameState, actions: &mut ActionQueue) {
        if self.state == TatakuTaskState::NotStarted {
            self.state = TatakuTaskState::Running;
        }

        if let Some(action) = self.action.take() {
            actions.push(action)
        }

        if self.action.is_none() {
            self.state = TatakuTaskState::Complete;
        }
    }
}
