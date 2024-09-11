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

// async trait
pub use async_trait::async_trait;

// triple buffer imports
#[cfg(feature = "ui")]
pub use triple_buffer::TripleBuffer;
#[cfg(feature = "ui")]
pub use triple_buffer::Input as TripleBufferSender;
#[cfg(feature = "ui")]
pub use triple_buffer::Output as TripleBufferReceiver;

pub use crossbeam::sync::{ ShardedLock, ShardedLockReadGuard, ShardedLockWriteGuard };
pub use global_value_manager::{ GlobalValue, GlobalValueManager, GlobalValueMut };

// winit imports
#[cfg(feature="graphics")]
pub use winit::event::MouseButton;

// tokio imports
pub use tokio::sync::{ OnceCell, Mutex as AsyncMutex, RwLock as AsyncRwLock };
pub use tokio::sync::mpsc::{UnboundedSender as AsyncUnboundedSender, UnboundedReceiver as AsyncUnboundedReceiver, unbounded_channel as async_unbounded_channel};
pub use tokio::sync::mpsc::{Sender as AsyncSender, Receiver as AsyncReceiver, channel as async_channel};

pub use parking_lot::{ Mutex, RwLock };

// serde imports
pub use serde::{ Serialize, Deserialize };

#[cfg(feature = "gameplay")]
pub use gilrs::{ Axis, Button as ControllerButton, GamepadId };

#[cfg(feature="graphics")]
pub use iced::advanced::graphics::core as iced_core;

// logging
pub use tracing::*;

// general game imports
pub use crate::game::*;
pub use crate::tasks::*;
#[cfg(feature="graphics")]
pub use crate::menus::*;
pub use crate::helpers::*;
pub use crate::managers::*;
pub use crate::integrations::*;

// tataku-client imports
pub use tataku_engine::prelude::*;