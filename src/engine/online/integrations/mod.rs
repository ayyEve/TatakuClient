mod lastfm;

#[cfg(feature="discord")]
mod discord;
#[cfg(not(feature="discord"))]
mod discord_nobuild;

pub use lastfm::*;

#[cfg(feature="discord")]
pub use discord::*;
#[cfg(not(feature="discord"))]
pub use discord_nobuild::*;
