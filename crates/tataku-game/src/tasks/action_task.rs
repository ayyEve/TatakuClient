use crate::prelude::*;

/// Just runs the action, primarily used with the DelayedTask to run actions after a certain amount of time
pub struct ActionTask {
    state: TatakuTaskState,
    action: Option<ActionTaskAction>,
}
impl ActionTask {
    pub fn new(action: impl Into<ActionTaskAction>) -> Self {
        Self {
            action: Some(action.into()),
            state: TatakuTaskState::NotStarted,
        }
    }
}

// overengineered lol
impl TatakuTask for ActionTask {
    fn get_name(&self) -> CowStr { "action_task".into() }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }
    
    fn run(
        &mut self, 
        values: &mut dyn Reflect, 
        _state: &TaskGameState, 
        actions: &mut ActionQueue
    ) {
        if self.state == TatakuTaskState::NotStarted {
            self.state = TatakuTaskState::Running;
        }

        if let Some(action) = self.action.take() {
            let action = match action {
                ActionTaskAction::Buildable {
                    action,
                    node,
                    passed_in
                } => action
                    .into_action(node, values, passed_in.as_ref())
                    .unwrap_or(TatakuAction::None),

                ActionTaskAction::Callback(cb) 
                    => cb(values),
                
                ActionTaskAction::Action(a) => a,
            };
            actions.push(action);
        }

        if self.action.is_none() {
            self.state = TatakuTaskState::Complete;
        }
    }
}


pub enum ActionTaskAction {
    Action(TatakuAction), 
    Callback(Box<dyn FnOnce(&mut dyn Reflect) -> TatakuAction + Send + Sync>),
    Buildable {
        action: BuildableAction,
        node: NodeId,
        passed_in: Option<TatakuValue>
    },
}
// impl<T: Into<TatakuAction>> From<T> for ActionTaskAction {
//     fn from(value: T) -> Self {
//         Self::Action(value.into())
//     }
// }
impl From<DelayedActionType> for ActionTaskAction {
    fn from(value: DelayedActionType) -> Self {
        match value {
            DelayedActionType::Action(a) => Self::Action(*a),
            DelayedActionType::Callback(cb) 
                => Self::Callback(cb),
        }
    }
}
