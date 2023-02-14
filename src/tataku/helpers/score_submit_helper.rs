use crate::prelude::*;

pub struct ScoreSubmitHelper {
    pub replay: Replay,
    settings: Settings,
    pub response: AsyncRwLock<Option<SubmitResponse>>,
}

impl ScoreSubmitHelper {
    pub fn new(replay: Replay, settings: &Settings) -> Arc<Self> {
        Arc::new(Self { replay, settings: settings.clone(), response: AsyncRwLock::new(None) })
    } 

    pub fn submit(self: Arc<Self>) {
        tokio::spawn(async move {
            trace!("submitting score");
            let replay = &self.replay;
            let score = replay.score_data.as_ref().unwrap();

            let username = self.settings.username.clone();
            let password = self.settings.password.clone();
            if username.is_empty() || password.is_empty() { 
                warn!("no user or pass, not submitting score");
                *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::NoUser, "No user/password in settings".to_owned()));
                return
            }


            let map = match BEATMAP_MANAGER.read().await.beatmaps_by_hash.get(&score.beatmap_hash) {
                None => { // what
                    *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::MapNotFound, "Map not found locally".to_owned()));
                    return 
                }
                Some(map) => map.clone()
            };
            let game = match &map.beatmap_type {
                BeatmapType::Osu => MapGame::Osu,
                BeatmapType::Quaver => MapGame::Quaver,
                other => MapGame::Other(format!("{other:?}").to_lowercase())
            };
            let map_info = ScoreMapInfo {
                game,
                map_hash: score.beatmap_hash.clone(),
                playmode: score.playmode.clone(),
            };
            let score_submit = ScoreSubmit {
                username,
                password,
                game: "tataku".to_owned(),
                replay: replay.clone(),
                map_info
            };

            if let Ok(replay_data) = serde_json::to_string(&score_submit) {
                let url = format!("{}/score_submit", self.settings.score_url);
                
                let c = reqwest::Client::new();
                let res = c
                    .post(url)
                    .header("Content-Type", "application/json")
                    .body(replay_data)
                    .send()
                    .await;

                match res {
                    Ok(resp) => {
                        if let Ok(txt) = resp.text().await {
                            info!("got score submit response: {txt}");

                            if let Some(resp) = serde_json::from_str::<SubmitResponse>(&txt).log_error().ok() {
                                // trace!("score submitted successfully");
                                *self.response.write().await = Some(resp);
                            } else {
                                *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::InternalError, "Error reading server response".to_owned()));
                            }
                        }
                    },
                    Err(e) => NotificationManager::add_error_notification("error submitting score", format!("{e}")).await,
                };
            } else {
                *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::InternalError, "Error serializing replay".to_owned()));
            }
            
        });
    }

}
