use crate::prelude::*;


pub struct AudioPreviewTask {
    preview_url: String,
    data_loader: Option<AsyncLoader<TatakuResult<Vec<u8>>>>,
    state: TatakuTaskState,
}

impl AudioPreviewTask {
    pub fn new(url: String) -> Self {
        Self {
            preview_url: url,
            data_loader: None,
            state: TatakuTaskState::NotStarted,
        }
    }

    async fn run_get(url: String) -> TatakuResult<Vec<u8>> {
        let bytes = reqwest::get(url)
            .await?
            .error_for_status()?
            .bytes()
            .await?
            ;
        Ok(bytes.to_vec())
    }
}

impl TatakuTask for AudioPreviewTask {
    fn get_name(&self) -> CowStr { format!("AudioPreview({})", self.preview_url).into() }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(
        &mut self, 
        _values: &mut dyn Reflect, 
        _state: &TaskGameState, 
        actions: &mut ActionQueue
    ) {
        // if we havent started yet, setup the data loader
        if self.data_loader.is_none() {
            let url = self.preview_url.clone();
            self.data_loader = Some(AsyncLoader::new(async move {
                Self::run_get(url).await
            }));

            self.state = TatakuTaskState::Running;
            return;
        }

        // we need a loader at this point
        let Some(loader) = &self.data_loader 
        else {
            self.state = TatakuTaskState::Complete;
            return
        };

        // if we have a result, deal with it, otherwise we're still waiting
        let Some(result) = loader.check() 
        else { return };

        match result {
            Ok(data) => {
                actions.push(SongAction::Set(SongSetAction::FromData(
                    data, 
                    self.preview_url.clone().into(),
                    SongPlayData {
                        play: true,
                        restart: true,
                        ..Default::default()
                    } 
                )));
            }

            Err(e) => {
                actions.push(Notification::new_error(
                    "Error loading audio preview", 
                    e
                ));
            }
        }

        self.state = TatakuTaskState::Complete;
    }
}
