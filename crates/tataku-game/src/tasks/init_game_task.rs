use crate::prelude::*;

#[derive(Default)]
pub struct InitGameTask {
    state: TatakuTaskState,

}


impl TatakuTask for InitGameTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Initialize Game") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(
        &mut self,
        values: &mut dyn Reflect, 
        _state: &TaskGameState, 
        actions: &mut ActionQueue,
    ) {
        let statuses = values
            .reflect_get_mut::<Vec<LoadingStatus>>("game.loading_statuses")
            .unwrap();

        if matches!(self.state, TatakuTaskState::NotStarted) {
            self.state = TatakuTaskState::Running;

            // load fonts
            preload_fonts();

            // load beatmaps
            statuses.push(LoadingStatus::new("Loading beatmaps"));
            actions.push(LoadBeatmapsTask::new(0));
        }

        
        // wait for all tasks to complete
        if statuses.iter().all(|s| s.complete) {
            self.state = TatakuTaskState::Complete;
            
            info!("game init done, going to main menu");
            actions.push(BeatmapAction::Next);
            actions.push(MenuAction::SetMenu { 
                id: "main_menu".into(), 
                input: BuildableInputArguments::default() 
            });
        }
    }
}