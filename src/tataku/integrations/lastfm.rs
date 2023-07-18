use crate::prelude::*;

pub struct LastFmIntegration;

impl LastFmIntegration {
    pub async fn check(settings: &Settings) {
        let username = settings.username.clone();
        let password = settings.password.clone();
        let url = settings.score_url.clone();
        drop(settings);

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

    pub async fn update(track:String, artist: String) {
        let settings = Settings::get();
        let username = settings.username.clone();
        let password = settings.password.clone();
        let url = settings.score_url.clone();
        drop(settings);


        let body = serde_json::to_string(&LastFmNowPlayingRequest { username, password, track, artist }).unwrap();
        let Ok(_) = reqwest::Client::new()
            .post(format!("{url}/lastfm/set_now_playing"))
            .header("Content-Type", "application/json")
            .body(body)
            .send().await else { return };

        // info!("{}", req.text().await.unwrap())
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
