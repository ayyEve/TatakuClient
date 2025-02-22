mod gl_error;
mod audio_error;
mod taiko_error;
mod beatmap_error;

pub use gl_error::*;
pub use audio_error::*;
pub use taiko_error::*;
pub use beatmap_error::*;


pub trait LogError {
    fn log_error(self) -> Self;
    fn log_error_message(self, msg: &str) -> Self;
}

// impl<T> LogError for TatakuResult<T> {
//     fn log_error(&self) -> &Self {
//         if let Err(e) = self {
//             error!("error: {e}")
//         }
//         self
//     }
//     fn log_error_message(&self, msg: &str) -> &Self {
//         if let Err(e) = self {
//             error!("{msg}: {e}")
//         }
//         self
//     }
// }

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
