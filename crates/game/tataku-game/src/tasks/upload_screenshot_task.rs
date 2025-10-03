use crate::prelude::*;
use tataku::Color;
use engine::{
    task::*,
    actions,
    notifications::{
        Notification,
        NotificationOnClick,
    },
};


pub struct UploadScreenshotTask {
    state: TatakuTaskState,
    data: ScreenshotData,

    task: Option<engine::AsyncLoader<Result<String, Notification>>>,
}
impl UploadScreenshotTask {
    pub fn new(data: impl Into<ScreenshotData>) -> Self {
        Self {
            state: TatakuTaskState::NotStarted,
            data: data.into(),
            task: None
        }
    }

    async fn upload(
        score_url: String, 
        username: String, 
        password: String,
        data: ScreenshotData,
    ) -> Result<String, Notification> {
        let url = format!("{score_url}/screenshots?username={username}&password={password}");

        let data = match data {
            ScreenshotData::Raw(data) => data,
            ScreenshotData::Path(path) => tataku::fs::read_file(path)
                .map_err(|e| Notification::new_error("Error loading screenshot to send to server", e))?,
        };

        let r = reqwest::Client::new().post(url).body(data).send().await
            .map_err(|e| Notification::new_error("Error sending screenshot request", e.to_string()))?;

        let s = r.text().await
            .map_err(|e| Notification::new_error("Error reading screenshot response", e.to_string()))?;

        let id = s.parse::<i64>()
            .map_err(|e | Notification::new_error("Error parsing screenshot id", e.to_string()))?;

        // copy to clipboard
        Ok(format!("{score_url}/screenshots/{id}"))
    }
}

impl TatakuTask for UploadScreenshotTask {
    fn get_name(&self) -> CowStr { Cow::Borrowed("Upload Screenshot") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(&mut self, shell: &mut TaskShell) {
        let Some(task) = self.task.as_ref() else {
            self.state = TatakuTaskState::Running;

            shell.actions.push(actions::game::GameAction::AddNotification(Notification::new_text(
                "Uploading screenshot...", Color::YELLOW, 5000.0
            )).into());

            let data = self.data.clone();
            let settings = shell.values
                .reflect_get::<engine::Settings>("settings")
                .unwrap();
            let connection = settings.connection().clone();

            self.task = Some(engine::AsyncLoader::new(Self::upload(
                connection.score_url,
                connection.tataku_username,
                connection.tataku_password,
                data,
            )));

            return;
        };

        let Some(received) = task.check() else { return };

        match received {
            Ok(url) => {
                shell.actions.push(Notification::new(
                    format!("Screenshot uploaded {url}"), 
                    Color::BLUE, 
                    5000.0, 
                    NotificationOnClick::Url(url.clone())
                ).into());
                shell.actions.push(actions::game::GameAction::CopyToClipboard(url.into()).into());
            }
            Err(notif) => shell.actions.push(notif.into()),
        }

        self.state = TatakuTaskState::Complete;
    }
}

#[derive(Clone)]
pub enum ScreenshotData {
    Raw(Vec<u8>),
    Path(String),
}
impl From<String> for ScreenshotData {
    fn from(value: String) -> Self {
        Self::Path(value)
    }
}
impl From<Vec<u8>> for ScreenshotData {
    fn from(value: Vec<u8>) -> Self {
        Self::Raw(value)
    }
}
