use crate::prelude::*;
use engine::task::*;

#[derive(Default2)]
pub(crate) struct TaskManager {
    tasks: Vec<TaskInner>,

    #[default(10)]
    max_tasks: usize,
}
impl TaskManager {
    pub fn add_task(&mut self, task: Box<dyn engine::Task>) {
        info!("Adding task: {}", task.get_name());

        self.tasks.push(TaskInner {
            task,
            // started: TatakuInstant::now(),
        });
    }

    pub fn update(&mut self, shell: &mut TaskShell) {
        let mut task_count = 0;

        // update our tasks
        for task in &mut self.tasks {
            // TODO: should we always run continuous tasks, even if our queue is full?
            // if so, we should make two task lists, one for continuous tasks, and one for one-time tasks
            // that way we dont need to iterate over all tasks to make sure all continuous tasks are run
            
            task_count += 1;
            if task_count > self.max_tasks { break }

            if task.get_state() == TatakuTaskState::NotStarted {
                info!("Starting task {}", task.get_name());
            }

            // run the task
            task.run(shell);

            if task.get_state() == TatakuTaskState::Complete {
                info!("Task complete {}", task.get_name());
            }

            // if task.get_type() == TatakuTaskType::Once && task.started.as_millis() > 60_000 {
            //     warn!("task has taken a long time: {}", task.get_name());
            // }
        }

        // remove any completed tasks
        self.tasks.retain(|t| t.get_state() != TatakuTaskState::Complete);
    }
}


struct TaskInner {
    /// What is this task?
    task: Box<dyn engine::Task>,

    // /// When did it start?
    // started: TatakuInstant,
}
impl Deref for TaskInner {
    type Target = Box<dyn engine::Task>;

    fn deref(&self) -> &Self::Target {
        &self.task
    }
}
impl DerefMut for TaskInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.task
    }
}
