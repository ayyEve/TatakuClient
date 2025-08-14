// std imports
pub use std::borrow::Cow;
pub use std::fmt::Display;
pub use std::time::Duration;
pub use std::f32::consts::PI;
pub use std::cell::{ Ref, RefCell };
pub use std::path::{ Path, PathBuf };
pub use std::ops::{ Range, Deref, DerefMut };
pub use std::collections::{ HashMap, HashSet, VecDeque };

pub use std::rc::Rc;
pub use std::sync::{ Arc, Weak };
pub use std::sync::atomic::{ *, Ordering::SeqCst };
pub use std::sync::mpsc::{ Sender, SyncSender, Receiver, sync_channel, channel };

pub type CowStr = Cow<'static, str>;

// async trait
pub use async_trait::async_trait;

// triple buffer imports
#[cfg(feature = "ui")]
pub use triple_buffer::{
    TripleBuffer,
    Input as TripleBufferSender,
    Output as TripleBufferReceiver
};


// tokio imports
pub use tokio::sync::{ OnceCell, Mutex as AsyncMutex, RwLock as AsyncRwLock };
pub use tokio::sync::mpsc::{ UnboundedSender as AsyncUnboundedSender, UnboundedReceiver as AsyncUnboundedReceiver, unbounded_channel as async_unbounded_channel };
pub use tokio::sync::mpsc::{ Sender as AsyncSender, Receiver as AsyncReceiver, channel as async_channel };

pub use parking_lot::{ Mutex, RwLock, MutexGuard };

// serde imports
pub use serde::{ Serialize, Deserialize };

// logging
pub use tracing::*;

// tataku imports
pub use tataku_common::types::*;
pub use tataku_input::prelude::*;
pub use tataku_common::prelude::*;
pub use tataku_client_proc_macros::*;
pub use tataku_client_common::prelude::*;

// folder imports
pub use crate::*;
pub use crate::io::*;
pub use crate::game::*;
pub use crate::data::*;
pub use crate::online::*;
pub use crate::window::*;
pub use crate::settings::*;
pub use crate::tataku_integration_event::*;
