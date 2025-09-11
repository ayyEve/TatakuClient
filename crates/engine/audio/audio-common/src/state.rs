use crate::*;
use crate::common::reflect::*;

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[reflect(display = "debug")]
pub enum AudioState {
    Playing,
    Paused,
    Stopped,
    
    #[default]
    Unknown,
}
