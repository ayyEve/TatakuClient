mod key;
mod event;
mod gamepad;
mod tataku_event;
mod mouse_button;
mod input_manager;
mod key_modifiers;

#[cfg(feature = "gameplay")]
pub use input_manager::*;
pub use key_modifiers::*;

pub mod prelude {
    #[allow(unused_imports)]
    pub(crate) use tracing::{ debug, info, warn, error };
    pub(crate) use std::collections::{ HashSet, HashMap };
    
    
    pub use gilrs;
    pub use gilrs::GamepadId;
    pub(crate) use serde::{ Serialize, Deserialize };


    pub(crate) use tataku_common::prelude::*;
    pub(crate) use tataku_client_common::prelude::*;


    pub use crate::key::*;
    pub use crate::event::*;
    pub use crate::gamepad::*;
    pub use crate::tataku_event::*;
    pub use crate::mouse_button::*;

    #[cfg(feature = "gameplay")]
    pub use crate::input_manager::*;
    pub use crate::key_modifiers::*;
}
