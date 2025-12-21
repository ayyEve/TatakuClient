pub mod io;
pub mod data;
pub mod math;
pub mod utils;
pub mod errors;
pub mod instant;
pub mod animate;
pub mod graphics;

pub mod prelude {
    pub use crate::common::*;

    pub use crate::errors::error::{
        Error, // tataku::Error
        TatakuResult as Result, // tataku::Result
    };

    pub use crate::io::*;
    pub use crate::errors;
    pub use crate::data::*;
    pub use crate::math::*;
    pub use crate::utils::*;
    pub use crate::instant::*;
    pub use crate::animate::*;
    pub use crate::graphics::*;
}

// things that should pretty much always be in scope
pub mod common {
    // re-exports
    pub use crate::errors;
    pub use crate::macros::*;
    pub use tataku_common as common;
    pub use crate::prelude as tataku;

    // std imports
    pub use std::path::Path;
    pub use std::borrow::Cow;
    pub use std::path::PathBuf;
    pub use std::sync::{ Arc, Weak };
    pub use std::ops::{ Deref, DerefMut };
    pub use std::collections::{ HashMap, HashSet };

    pub type CowStr = Cow<'static, str>;

    // external imports
    pub use half::f16;

    pub use tracing;
    pub use tracing::*;

    pub use bitflags::bitflags;
    pub use parking_lot::{ Mutex, RwLock, MutexGuard };

    pub use serde;

    // trait exports
    pub use crate::data::Nope;
    pub use crate::data::Take;
    pub use crate::data::ArcStr;
    pub use crate::math::Interpolation;
    pub use crate::math::MatrixHelpers;
}

// proc macro exports
pub mod macros {
    pub use serde::Serialize;
    pub use serde::Deserialize;
    pub use tataku_common::macros::*;
    pub use tataku_client_proc_macros::*;
}
