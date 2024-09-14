use crate::prelude::*;

pub struct ScoreSubmitHelper {
    pub score: Score,
    // settings: Settings,
    username: String,
    password: String,
    score_url: String,

    beatmap_type: BeatmapType,
    pub response: AsyncRwLock<Option<SubmitResponse>>,
}

impl ScoreSubmitHelper {
    pub fn new(
        score: Score, 
        settings: &Settings,
        beatmap: &BeatmapMeta,
    ) -> Arc<Self> {
        Arc::new(Self { 
            score, 
            // settings: settings.clone(), 
            beatmap_type: beatmap.beatmap_type,
            username: settings.username.clone(),
            password: settings.password.clone(),
            score_url: settings.score_url.clone(),
            response: AsyncRwLock::new(None),
        })
    } 

    pub fn submit(self: Arc<Self>) {
        tokio::spawn(async move {
            trace!("submitting score");

            let username = self.username.clone();
            let password = self.password.clone();
            if username.is_empty() || password.is_empty() { 
                warn!("no user or pass, not submitting score");
                *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::NoUser, "No user/password in settings".to_owned()));
                return
            }


            // let map = match BEATMAP_MANAGER.read().await.beatmaps_by_hash.get(&score.beatmap_hash) {
            //     None => { // what? how did you play the map then?
            //         *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::MapNotFound, "Map not found locally".to_owned()));
            //         return 
            //     }
            //     Some(map) => map.clone()
            // };
            let map_info = ScoreMapInfo {
                game: self.beatmap_type.into(),
                map_hash: self.score.beatmap_hash,
                playmode: self.score.playmode.clone(),
            };
            let score_submit = ScoreSubmit {
                username,
                password,
                game: "tataku".to_owned(),
                score: self.score.clone(),
                map_info
            };

            if let Ok(replay_data) = serde_json::to_string(&score_submit) {
                let url = format!("{}/score_submit", self.score_url);
                
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

                            match serde_json::from_str::<SubmitResponse>(&txt) {
                                Ok(resp) => {
                                    // trace!("score submitted successfully");
                                    *self.response.write().await = Some(resp)
                                }
                                Err(e) => {
                                    error!("{e}");
                                    *self.response.write().await = Some(SubmitResponse::NotSubmitted(NotSubmittedReason::InternalError, "Error reading server response".to_owned()));
                                }
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
