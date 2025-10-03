use crate::prelude::*;
use engine::game::task::*;
use common::{
    ScoreSubmit,
    ScoreMapInfo,
    SubmitResponse,
    NotSubmittedReason,
    reflect::*,
};

fn score_submit_path() -> String {
    const SCORE_SUBMIT_REFLECT_PATH: &str = "var.score_submits";
    /// SAFETY: uhhhhhhhhh its fine
    static mut SCORE_SUBMIT_COUNTER: u16 = 0;

    let path = format!("{SCORE_SUBMIT_REFLECT_PATH}.{}", unsafe {SCORE_SUBMIT_COUNTER});
    unsafe { SCORE_SUBMIT_COUNTER += 1; }

    path
}

pub struct UploadScoreTask {
    state: TatakuTaskState,
    data: ScoreUploadData,
    task: Option<engine::io::AsyncLoader<SubmitResponse>>,
}
impl UploadScoreTask {
    pub fn new(
        score: common::Score, 
        beatmap: &engine::BeatmapMeta,
        settings: &engine::Settings,
    ) -> Self {
        let connection = settings.connection().clone();

        Self {
            state: TatakuTaskState::NotStarted,
            data: ScoreUploadData { 
                score_submit: ScoreSubmit {
                    username: connection.tataku_username.clone(),
                    password: connection.tataku_password.clone(),
                    game: "tataku".to_owned(),
                    map_info: ScoreMapInfo {
                        game: beatmap.beatmap_type.into(),
                        map_hash: score.beatmap_hash,
                        playmode: score.playmode.clone(),
                    },
                    score,
                }, 
                score_url: connection.score_url.clone(),
                path: score_submit_path(),
                // delay: 0
            },
            task: None,
        }
    }

    pub fn get_path(&self) -> &str {
        &self.data.path
    }

    async fn upload(
        data: ScoreUploadData,
    ) -> SubmitResponse {
        trace!("submitting score");
        if data.score_submit.username.is_empty() || data.score_submit.password.is_empty() { 
            warn!("no user or pass, not submitting score");
            return SubmitResponse::NotSubmitted(NotSubmittedReason::NoUser, "No user/password in settings".to_owned());
        }

        if let Ok(replay_data) = serde_json::to_string(&data.score_submit) {
            let url = format!("{}/score_submit", data.score_url);
            
            let c = reqwest::Client::new();
            let res = c
                .post(url)
                .header("Content-Type", "application/json")
                .body(replay_data)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    let txt = resp.text().await.unwrap();
                    info!("got score submit response: {txt}");

                    match serde_json::from_str::<SubmitResponse>(&txt) {
                        Ok(resp) => resp,
                        Err(e) => {
                            error!("{e}");
                            SubmitResponse::NotSubmitted(NotSubmittedReason::InternalError, "Error reading server response".to_owned())
                        }
                    }
                }

                Err(e) => SubmitResponse::NotSubmitted(NotSubmittedReason::InternalError, format!("error submitting score: {e}")),
            }
        } else {
            SubmitResponse::NotSubmitted(NotSubmittedReason::InternalError, "Error serializing replay".to_owned())
        }
        
    }
}

impl TatakuTask for UploadScoreTask {
    fn get_name(&self) -> CowStr { Cow::Borrowed("Upload Score") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(&mut self, shell: &mut TaskShell) {
        let Some(task) = self.task.as_ref() else {
            self.state = TatakuTaskState::Running;

            let data = self.data.clone();
            self.task = Some(engine::io::AsyncLoader::new(Self::upload(data)));

            shell.values.reflect_insert(self.get_path(), ScoreSubmitResponse::default()).unwrap();

            return;
        };

        let Some(received) = task.check() else { return };

        match received {
            SubmitResponse::Submitted {
                score_id,
                placing,
                performance_rating,
            } => {
                let a = shell.values.reflect_get_mut::<ScoreSubmitResponse>(self.get_path()).unwrap();
                a.completed = true;
                a.score_id = score_id;
                a.placing = placing;
                a.performance_rating = performance_rating;
            }
            SubmitResponse::NotSubmitted(_e, msg) => {
                shell.actions.push(engine::Notification::new_error(
                    "Error submitting score", 
                    msg
                ).into());
                
                let a = shell.values
                    .reflect_get_mut::<ScoreSubmitResponse>(self.get_path())
                    .unwrap();
                a.completed = true;

                // TODO: try again?
                // match e {
                //     NotSubmittedReason::InternalError => todo!(),
                //     NotSubmittedReason::NoUser => todo!(),
                //     NotSubmittedReason::UserBanned => todo!(),
                //     NotSubmittedReason::MapNotFound => todo!(),
                //     NotSubmittedReason::GameNotAccepted => todo!(),
                //     NotSubmittedReason::Other(_) => todo!(),
                // }
            }
        }

        self.state = TatakuTaskState::Complete;
    }
}

#[derive(Clone)]
struct ScoreUploadData {
    score_submit: ScoreSubmit,
    score_url: String,
    path: String,
    // delay: u32,
}

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Copy, Clone, Debug, Default)]
pub struct ScoreSubmitResponse {
    pub completed: bool,
    pub score_id: u64,
    pub placing: u32,
    pub performance_rating: f32,
}
