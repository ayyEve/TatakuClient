mod audio_error;
mod tataku_error;
mod beatmap_error;
mod graphics_error;
mod download_error;

pub use audio_error::*;
pub use tataku_error::*;
pub use beatmap_error::*;
pub use graphics_error::*;
pub use download_error::*;


pub trait LogError {
    fn log_error(self) -> Self;
    fn log_error_message(self, msg: &str) -> Self;
}

impl<T, E:ToString> LogError for Result<T, E> {
    fn log_error(self) -> Self {
        if let Err(e) = &self {
            error!("error: {}", e.to_string())
        }
        self
    }
    fn log_error_message(self, msg: &str) -> Self {
        if let Err(e) = &self {
            error!("{msg}: {}", e.to_string())
        }
        self
    }
}
