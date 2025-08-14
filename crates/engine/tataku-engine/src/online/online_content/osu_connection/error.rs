pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}