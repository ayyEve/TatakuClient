use serde::{ Serialize, Deserialize };
use tataku_engine::{
    CowStr,
    ArcStr,
    tataku,
    actions,
    Settings,
    tataku::open_link,
    TatakuIntegrationEvent,
    common::reflect::Reflect,
    io::{
        TatakuIntegration,
        TatakuIntegrationBuilder
    },
};

pub struct LastFm;
impl LastFm {
    fn build() -> tataku::Result<Box<dyn TatakuIntegration>> {
        Ok(Box::new(Self))
    }

    pub fn builder() -> TatakuIntegrationBuilder {
        TatakuIntegrationBuilder {
            name: "LastFM",
            build: Self::build
        }
    }

    pub async fn check(settings: &Settings) {
        let connection = settings.connection().clone();

        let username = connection.tataku_username;
        let password = connection.tataku_password;
        let url = connection.score_url;

        let body = serde_json::to_string(&LastFmAuthRequest { username, password }).unwrap();
        let Ok(req) = reqwest::Client::new()
            .post(format!("{url}/lastfm/check"))
            .header("Content-Type", "application/json")
            .body(body)
            .send().await else { return };

        let txt = req.text().await.unwrap();
        if let Ok(resp) = serde_json::from_str::<LastFMAuthReponse>(&txt)
        && let Some(url) = resp.auth_url { 
            open_link(url); 
        }
    }

    pub async fn update(track: ArcStr, artist: ArcStr, settings: &Settings) {
        let connection = settings.connection().clone();
        let url = connection.score_url;

        let body = serde_json::to_string(&LastFmNowPlayingRequest { 
            username: connection.tataku_username.into(), 
            password: connection.tataku_password.into(), 
            track, 
            artist 
        }).unwrap();
        let Ok(_) = reqwest::Client::new()
            .post(format!("{url}/lastfm/set_now_playing"))
            .header("Content-Type", "application/json")
            .body(body)
            .send().await else { return };
    }
}
impl TatakuIntegration for LastFm {
    fn name(&self) -> CowStr { "LastFm".into() }

    fn init(
        &mut self, 
        #[cfg(feature="graphics")] 
        _window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> tataku::Result<()> {
        Ok(())
    }

    fn check_enabled(
        &mut self, 
        _settings: &Settings
    ) -> tataku::Result<()> {
        Ok(())
    }

    fn handle_event(
        &mut self, 
        event: &TatakuIntegrationEvent,
        values: &dyn Reflect,
        _actions: &mut actions::ActionQueue,
    ) {
        let TatakuIntegrationEvent::SongChanged { 
            artist, 
            title, 
            .. 
        } = event else { return };
        let settings = values.reflect_get::<Settings>("settings").unwrap();

        let track = title.clone();
        let artist = artist.clone();

        let connection = settings.connection().clone();
        tokio::spawn(async move {
            let url = connection.score_url;
            let body = serde_json::to_string(&LastFmNowPlayingRequest { 
                username: connection.tataku_username.into(), 
                password: connection.tataku_password.into(), 
                track, 
                artist 
            }).unwrap();
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
    username: ArcStr,
    password: ArcStr,
    artist: ArcStr,
    track: ArcStr,
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
