use crate::prelude::*;

pub struct UploadScreenshotTask {
    state: TatakuTaskState,
    screenshot_path: String,

    task: Option<AsyncLoader<Result<String, Notification>>>,
}
impl UploadScreenshotTask {
    pub fn new(screenshot_path: String) -> Self {
        Self {
            state: TatakuTaskState::NotStarted,
            screenshot_path,
            task: None
        }
    }

    async fn upload(
        score_url: String, 
        username: String, 
        password: String,
        screenshot_path: String,
    ) -> Result<String, Notification> {
        let url = format!("{score_url}/screenshots?username={username}&password={password}");

        let data = Io::read_file_async(screenshot_path).await
            .map_err(|e| Notification::new_error("Error loading screenshot to send to server", e))?;

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

#[async_trait]
impl TatakuTask for UploadScreenshotTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Upload Screentho") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(&mut self, values: &mut dyn Reflect, _: &TaskGameState, actions: &mut ActionQueue) {
        let Some(task) = self.task.as_ref() else {
            self.state = TatakuTaskState::Running;

            actions.push(GameAction::AddNotification(Notification::new_text(
                "Uploading screenshot...", Color::YELLOW, 5000.0
            )));

            let settings = values.reflect_get::<Settings>("settings").unwrap();
            self.task = Some(AsyncLoader::new(Self::upload(
                settings.score_url.clone(),
                settings.username.clone(),
                settings.password.clone(),
                self.screenshot_path.clone(),
            )));

            return;
        };

        let Some(received) = task.check().await else { return };

        match received {
            Ok(url) => {
                actions.push(GameAction::AddNotification(Notification::new(
                    format!("Screenshot uploaded {url}"), 
                    Color::BLUE, 
                    5000.0, 
                    NotificationOnClick::Url(url.clone())
                )));
                actions.push(GameAction::CopyToClipboard(url));
            }
            Err(notif) => actions.push(GameAction::AddNotification(notif)),
        }

        self.state = TatakuTaskState::Complete;
    }
}