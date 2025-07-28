// TODO: if request fails due to expired token, get new token and try again
use crate::prelude::*;
use super::{
    token::*,
    consts::*,
};

#[derive(Default)]
pub struct OsuConnection {
    token: Token,
}
impl OsuConnection {
    pub fn setup_failable(settings: &Settings) -> TatakuResult<Self> {
        let p = std::path::Path::new(REFRESH_TOKEN_FILE);

        let mut token = None;
        if let Ok(refresh_token) = std::fs::read_to_string(p) {
            if let Ok(refreshed_token) = Token::refresh(refresh_token) {
                token = Some(refreshed_token);
            }
        }
        let token = if let Some(token) = token {
            token
        } else {
            Token::authenticate(settings)?
        };
        std::fs::write(p, &token.refresh_token)?;
        
        Ok(Self {
            token,
        })
    }

    pub fn setup(settings: &Settings) -> Option<Self> {
        for _ in 0..5 {
            let Ok(a) = Self::setup_failable(settings)
                .inspect_err(|e| error!("{e:?}")) 
            else { continue };

            return Some(a);
        }

        None
    }

    pub fn bearer(&self) -> String {
        self.token.bearer()
    }
}
