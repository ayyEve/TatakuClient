use crate::prelude::*;

pub struct TaskManager {
    tasks: Vec<TaskInner>,

    max_tasks: usize,
}
impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),

            max_tasks: 10,
        }
    }

    pub fn add_task(&mut self, task: Box<dyn TatakuTask>) {
        info!("Adding task: {}", task.get_name());

        self.tasks.push(TaskInner {
            task,
            started: Instant::now(),
        })
    }

    pub async fn update(&mut self, values: &mut ValueCollection, state: TaskGameState) -> Vec<TatakuAction> {
        let mut actions = ActionQueue::new();
        let mut task_count = 0;

        // update our tasks
        for task in &mut self.tasks {
            // TODO: should we always run continuous tasks, even if our queue is full?
            // if so, we should make two task lists, one for continuous tasks, and one for one-time tasks
            // that way we dont need to iterate over all tasks to make sure all continuous tasks are run
            
            task_count += 1;
            if task_count > self.max_tasks { break }

            // run the task
            task.run(values, &state, &mut actions).await;

            // if task.get_type() == TatakuTaskType::Once && task.started.as_millis() > 60_000 {
            //     warn!("task has taken a long time: {}", task.get_name());
            // }
        }

        // remove any completed tasks
        self.tasks.retain(|t| t.get_state() != TatakuTaskState::Complete);

        actions.take()
    }
}



struct TaskInner {
    /// What is this task?
    task: Box<dyn TatakuTask>,

    /// When did it start?
    started: Instant,
}
impl Deref for TaskInner {
    type Target = Box<dyn TatakuTask>;

    fn deref(&self) -> &Self::Target {
        &self.task
    }
}
impl DerefMut for TaskInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task
    }
}
