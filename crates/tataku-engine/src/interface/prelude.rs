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

// tataku-common imports
pub use tataku_common::types::*;

// folder imports
pub use crate::cursor::*;
pub use crate::skinning::*;
pub use crate::ui_element::*;
pub use crate::fps_display::*;
#[cfg(feature="graphics")]
pub use crate::menu_elements::*;
pub use crate::visualizations::*;
pub use crate::generic_button::*;
pub use crate::volume_control::*;
pub use crate::cursor_manager::*;
pub use crate::ingame_elements::*;
pub use crate::centered_text_helper::*;

pub use tataku_client_proc_macros::*;
pub use tataku_engine::prelude::*;

// online imports
pub use tataku_common::packets::*;
pub use tataku_common::reflection::*;
pub use tataku_common::serialization::*;
pub use tataku_common_proc_macros::Reflect;


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