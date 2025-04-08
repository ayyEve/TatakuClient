use serde::Serialize;
use tataku_engine::prelude::*;

pub struct LastFm;
impl LastFm {
    fn build() -> TatakuResult<Box<dyn TatakuIntegration>> {
        Ok(Box::new(Self))
    }

    pub fn builder() -> TatakuIntegrationBuilder {
        TatakuIntegrationBuilder {
            name: "LastFM",
            build: Self::build
        }
    }

    pub async fn check(settings: &Settings) {
        let username = settings.username.clone();
        let password = settings.password.clone();
        let url = settings.score_url.clone();

        let body = serde_json::to_string(&LastFmAuthRequest { username, password }).unwrap();
        let Ok(req) = reqwest::Client::new()
            .post(format!("{url}/lastfm/check"))
            .header("Content-Type", "application/json")
            .body(body)
            .send().await else { return };

        let txt = req.text().await.unwrap();
        if let Ok(resp) = serde_json::from_str::<LastFMAuthReponse>(&txt) {
            if let Some(url) = resp.auth_url { open_link(url); }
        }
    }

    pub async fn update(track: String, artist: String, settings: &Settings) {
        let username = settings.username.clone();
        let password = settings.password.clone();
        let url = settings.score_url.clone();

        let body = serde_json::to_string(&LastFmNowPlayingRequest { username, password, track, artist }).unwrap();
        let Ok(_) = reqwest::Client::new()
            .post(format!("{url}/lastfm/set_now_playing"))
            .header("Content-Type", "application/json")
            .body(body)
            .send().await else { return };
    }
}
impl TatakuIntegration for LastFm {
    fn name(&self) -> Cow<'static, str> { "LastFm".into() }

    fn init(
        &mut self, 
        _window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> TatakuResult<()> {
        Ok(())
    }

    fn check_enabled(
        &mut self, 
        _settings: &Settings
    ) -> TatakuResult<()> {
        Ok(())
    }

    fn handle_event(
        &mut self, 
        event: &TatakuIntegrationEvent,
        values: &dyn Reflect,
        _actions: &mut ActionQueue,
    ) {
        let TatakuIntegrationEvent::SongChanged { artist, title, .. } = event else { return };
        let settings = values.reflect_get::<Settings>("settings").unwrap();

        let track = title.clone();
        let artist = artist.clone();

        let username = settings.username.clone();
        let password = settings.password.clone();
        let url = settings.score_url.clone();

        tokio::spawn(async move {
            let body = serde_json::to_string(&LastFmNowPlayingRequest { username, password, track, artist }).unwrap();
            let Ok(_) = reqwest::Client::new()
                .post(format!("{url}/lastfm/set_now_playing"))
                .header("Content-Type", "application/json")
                .body(body)
                .send().await else { return };
        });
    }

}

#[derive(Serialize)]
struct LastFmNowPlayingRequest {
    username: String,
    password: String,
    artist: String,
    track: String
}

#[derive(Serialize)]
pub struct LastFmAuthRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LastFMAuthReponse {
    auth_url: Option<String>,
}
