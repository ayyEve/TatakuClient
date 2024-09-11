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

// tataku-common imports
pub use tataku_common::types::*;

// folder imports
pub use crate::io::*;
pub use crate::game::*;
pub use crate::data::*;
pub use crate::audio::*;
pub use crate::input::*;
pub use crate::online::*;
pub use crate::window::*;
pub use crate::locale::*;
pub use crate::settings::*;
pub use crate::graphics::*;
pub use crate::databases::*;
pub use crate::interface::*;
pub use crate::tataku_event::*;

// general game imports
pub use tataku_client_proc_macros::*;
pub use crate::*;

pub use tataku_common::prelude::*;
pub use tataku_client_common::prelude::*;


// iced imports, in its own mod since it has some comflicting names
#[cfg(feature="graphics")]
pub mod iced_elements {
    // macro imports
    pub use crate::row;
    pub use crate::col;

    // common structs/enums used by iced
    pub use iced::Length;
    pub use iced::Length::{Fill, FillPortion, Shrink, Fixed};
    pub use iced::Alignment;
    pub use iced::Rectangle;
    pub use iced::alignment::Horizontal;
    pub use iced::alignment::Vertical;

    // widgets
    pub use iced::widget::Row;
    pub use iced::widget::Text;
    pub use iced::widget::Space;
    pub use iced::widget::Button;
    pub use iced::widget::Column;
    pub use iced::widget::Checkbox;
    pub use iced::widget::TextInput;
    pub use iced::widget::Container;
    pub use iced::widget::Slider;
    pub use iced::widget::PickList as Dropdown;
}