use crate::*;
use super::consts::*;
use std::time::SystemTime;

#[derive(Default)]
#[allow(unused)]
pub struct Token {
    pub token_type: String,
    pub expires_at: u64,
    pub access_token: String,
    pub refresh_token: String,
}
impl Token {
    pub fn authenticate(settings: &Settings) -> tataku::Result<Self> {
        #[derive(Serialize)]
        struct Request {
            client_id: String,
            client_secret: String,
            username: String,
            password: String,
            grant_type: String, //"password"
            scope: String, // "*"
        }
        let osu_integration = settings.integrations.osu.clone();


        let response = ureq::post(TOKEN_URL)
            .content_type("application/json")
            .header("Accept", "application/json")
            .header("User-Agent", "osu!")
            .send(serde_json::to_string(&Request {
                client_id: LAZER_CLIENT_ID.to_owned(), // osu lazer's client id
                client_secret: LAZER_CLIENT_SECRET.to_owned(), // osu lazer's client secret
                username: osu_integration.username.clone(),
                password: osu_integration.password.clone(),
                grant_type: "password".to_owned(),
                scope: "*".to_owned(),
            }).unwrap())?;
        
        #[derive(Deserialize)]
        struct Response {
            token_type: String,
            expires_in: u64, // seconds
            access_token: String,
            refresh_token: String,
        }

        let Response {
            token_type,
            expires_in,
            access_token,
            refresh_token
        } = serde_json::from_slice(
            &response.into_body().read_to_vec()?
        )?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            token_type,
            expires_at: now + expires_in,
            access_token,
            refresh_token,
        })
    }

    pub fn refresh(refresh_token: String) -> tataku::Result<Self> {
        #[derive(Serialize)]
        struct Request {
            client_id: String,
            client_secret: String,
            grant_type: String, //"password"
            refresh_token: String,
        }

        let response = ureq::post(TOKEN_URL)
            .content_type("application/json")
            .header("Accept", "application/json")
            .header("User-Agent", "osu!")
            .send(serde_json::to_string(&Request {
                client_id: LAZER_CLIENT_ID.to_owned(), // osu lazer's client id
                client_secret: LAZER_CLIENT_SECRET.to_owned(), // osu lazer's client secret
                grant_type: "refresh_token".to_owned(),
                refresh_token,
            }).unwrap())?;

        #[derive(Deserialize)]
        struct Response {
            token_type: String,
            expires_in: u64, // seconds
            access_token: String,
            refresh_token: String
        }

        let Response {
            token_type,
            expires_in,
            access_token,
            refresh_token
        } = serde_json::from_slice(
            &response.into_body().read_to_vec()?
        )?;

        
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            token_type,
            expires_at: now + expires_in,
            access_token,
            refresh_token,
        })
    }

    pub fn bearer(&self) -> String {
        if self.token_type == "Bearer" {
            format!("{} {}", self.token_type, self.access_token)
        } else { panic!("invalid token type: {}", self.token_type); }
    }

    // pub fn expires_soon(&self) -> bool {
    //     Utc::now() > self.expires - Duration::from_secs(SECONDS_UNTIL_REFRESH)
    // }
}
