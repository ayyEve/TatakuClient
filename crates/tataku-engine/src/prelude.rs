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
pub use crate::audio::*;
pub use crate::online::*;
pub use crate::window::*;
pub use crate::locale::*;
pub use crate::settings::*;

#[cfg(feature="graphics")]
pub use crate::graphics::*;
pub use crate::tataku_event::*;
pub use crate::tataku_integration_event::*;

/// ui imports, in its own mod 
/// 
/// \* \~ **Organization!** \~ *
#[cfg(feature="ui")]
pub mod ui {
    pub use taffy::Size;
    pub use taffy::Style;
    pub use taffy::Layout;
    pub use taffy::Display;
    pub use taffy::Dimension;
    pub use taffy::TaffyTree;
    pub use taffy::TaffyResult;
    pub use taffy::AlignContent;
    pub use taffy::FlexDirection;
    pub use taffy::LengthPercentage;
    pub use taffy::LengthPercentageAuto;
    pub use taffy::NodeId as TaffyNodeId;
    
    pub use crate::graphics::ui::style::*;
    pub use crate::graphics::ui::operations::*;

    
    pub const EMPTY_NODE: super::NodeId = super::NodeId {
        node_id: TaffyNodeId::new(u64::MAX),
        owner: super::MessageOwner::Menu,
    };
    pub const FILL: Dimension = Dimension::Percent(1.0);
    pub const SHRINK: Dimension = Dimension::Auto;
    
    /// generic layout for menus
    pub fn menu_layout() -> Style {
        use taffy::prelude::*;
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            box_sizing: taffy::BoxSizing::ContentBox,
            position: taffy::Position::Relative,
            overflow: taffy::Point {
                x: taffy::Overflow::Hidden,
                y: taffy::Overflow::Hidden,
            },
            
            align_self: None,
            align_items: Some(AlignItems::Stretch),
            align_content: Some(AlignContent::SpaceBetween),

            justify_self: None,
            justify_items: Some(AlignItems::Stretch),
            justify_content: Some(AlignContent::SpaceBetween),
            
            size: Size::from_percent(1.0, 1.0),
            min_size: Size::from_percent(1.0, 1.0),
            max_size: Size::from_percent(1.0, 1.0),

            ..Default::default()
        }
    }

}

