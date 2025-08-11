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

// triple buffer imports
#[cfg(feature = "ui")]
pub use triple_buffer::{
    TripleBuffer,
    Input as TripleBufferSender,
    Output as TripleBufferReceiver,
};

// tokio imports
pub use tokio::sync::{ OnceCell, Mutex as AsyncMutex, RwLock as AsyncRwLock };
pub use tokio::sync::mpsc::{ UnboundedSender as AsyncUnboundedSender, UnboundedReceiver as AsyncUnboundedReceiver, unbounded_channel as async_unbounded_channel };
pub use tokio::sync::mpsc::{ Sender as AsyncSender, Receiver as AsyncReceiver, channel as async_channel };

// serde imports
pub use serde::{ Serialize, Deserialize };

// logging
pub use tracing::*;

// tataku imports
pub use crate::game::*;
pub use crate::tasks::*;
pub use crate::values::*;
pub use crate::managers::*;
pub use crate::database::*;
pub use tataku_engine::prelude::*;

#[cfg(feature="graphics")] pub use tataku_interface::prelude::*;
