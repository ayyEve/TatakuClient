// std imports
pub use std::borrow::Cow;
pub use std::fmt::Display;
pub use std::time::Duration;
pub use std::f32::consts::PI;
pub use std::path::{ Path, PathBuf };
pub use std::ops::{ Range, Deref, DerefMut };
pub use std::collections::{ HashMap, HashSet, VecDeque };

// sync imports
pub use std::sync::{ Arc, Weak };
pub use std::sync::atomic::{ *, Ordering::SeqCst };
pub use std::sync::mpsc::{ Sender, SyncSender, Receiver, sync_channel, channel };

// async trait
pub use async_trait::async_trait;

// triple buffer imports
pub use triple_buffer::TripleBuffer;
pub use triple_buffer::Input as TripleBufferSender;
pub use triple_buffer::Output as TripleBufferReceiver;

pub use crossbeam::sync::{ ShardedLock, ShardedLockReadGuard, ShardedLockWriteGuard };
pub use global_value_manager::{ GlobalValue, GlobalValueManager, GlobalValueMut };

// piston imports
pub use winit::event::MouseButton;
pub use winit::event::VirtualKeyCode as Key;

// tokio imports
pub use tokio::sync::{ OnceCell, Mutex as AsyncMutex, RwLock as AsyncRwLock };
pub use tokio::sync::mpsc::{UnboundedSender as AsyncUnboundedSender, UnboundedReceiver as AsyncUnboundedReceiver, unbounded_channel as async_unbounded_channel};
pub use tokio::sync::mpsc::{Sender as AsyncSender, Receiver as AsyncReceiver, channel as async_channel};

pub use parking_lot::{ Mutex, RwLock };

// serde imports
pub use serde::{ Serialize, Deserialize };

pub use gilrs::{ Axis, Button as ControllerButton, GamepadId };

pub use iced::advanced::graphics::core as iced_core;

// tataku-common imports
pub use tataku_common::types::*;

// folder imports
pub use crate::SONGS_DIR;
pub use crate::REPLAYS_DIR;
pub use crate::SKINS_FOLDER;
pub use crate::DOWNLOADS_DIR;

// general game imports
pub use crate::engine::*;
pub use crate::tataku::*;
pub use crate::interface::*;
pub use crate::tataku::modes::*;
pub use tataku_client_proc_macros::*;

// online imports
pub use tataku_common::packets::*;
pub use tataku_common::serialization::*;



// iced imports, in its own mod since it has some comflicting names
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