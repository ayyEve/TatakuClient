// when (if) i add online things, this is where they will be stored.

mod chat;
mod online_user;
pub mod discord;
mod online_manager;

pub use chat::*;
pub use online_user::*;
pub use online_manager::*;