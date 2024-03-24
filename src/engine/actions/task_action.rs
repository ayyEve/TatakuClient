use crate::prelude::*;


pub enum TaskAction {
    AddTask(Box<dyn TatakuTask>),
}

impl From<TaskAction> for TatakuAction {
    fn from(value: TaskAction) -> Self {
        Self::Task(value)
    }
}

impl std::fmt::Debug for TaskAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddTask(task) => write!(f, "AddTask({})", task.get_name()),
        }
    }
}