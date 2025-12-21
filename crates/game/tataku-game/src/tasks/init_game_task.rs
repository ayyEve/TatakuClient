use crate::prelude::*;
use engine::{
    actions,
    game::task::*,
};

#[derive(Default)]
pub struct InitGameTask {
    state: TatakuTaskState,
}

impl engine::Task for InitGameTask {
    fn get_name(&self) -> CowStr { Cow::Borrowed("Initialize Game") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(&mut self, shell: &mut TaskShell) {
        let values = ValueCollection::from_reflect_mut(shell.values);
        let statuses = &mut values.game.loading_statuses;

        if matches!(self.state, TatakuTaskState::NotStarted) {
            self.state = TatakuTaskState::Running;

            // load beatmaps
            statuses.push(LoadingStatus::new("Loading beatmaps"));
            shell.actions.push(LoadBeatmapsTask::new(0).into());
        }

        
        // wait for all tasks to complete
        if statuses.iter().all(|s| s.complete) {
            self.state = TatakuTaskState::Complete;
            
            info!("game init done, going to main menu");
            shell.actions.push(actions::beatmap::BeatmapAction::Next.into());
            
            #[cfg(feature="graphics")]
            shell.actions.push(actions::menu::MenuAction::SetMenu { 
                id: "main_menu".into(),
            }.into());
        }
    }
}
