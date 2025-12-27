mod key;
mod event;
mod gamepad;
mod tataku_event;
mod mouse_button;
mod input_manager;
mod key_modifiers;

// re-export smol_str for convenience
pub use smol_str;

#[cfg(feature = "gameplay")]
pub use input_manager::*;
pub use key_modifiers::*;

#[allow(unused_imports)]
pub(crate) use tracing::{ debug, info, warn, error };
pub(crate) use std::collections::{ HashSet, HashMap };


pub use gilrs;
pub use gilrs::GamepadId;
pub(crate) use serde::{ Serialize, Deserialize };
pub(crate) use tataku_engine_common::prelude::*;

pub use crate::key::*;
pub use crate::event::*;
pub use crate::gamepad::*;
pub use crate::tataku_event::*;
pub use crate::mouse_button::*;

