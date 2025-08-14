use crate::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[derive(Reflect)]
#[reflect(display = "debug")]
pub enum AudioState {
    Playing,
    Paused,
    Stopped,
    
    #[default]
    Unknown,
}

