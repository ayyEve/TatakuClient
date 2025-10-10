pub mod style;
pub mod message;
pub mod tree;
pub mod widget;
pub mod spatial_navigation;
mod current_input_state;


use crate::tree::*;
use crate::style::*;
use crate::message::*;


pub const EMPTY_NODE: NodeId = NodeId {
    node_id: taffy::NodeId::new(u64::MAX),
    source: MessageSource::Menu,
};
pub const FILL: CssUnit = CssUnit::Percent(f16::from_f32_const(1.0));
pub const SHRINK: CssUnit = CssUnit::Auto;

pub(crate) use std::str::FromStr;
pub(crate) use tataku_input as input;
pub(crate) use tataku_graphics as graphics;
pub(crate) use tataku_engine_common::prelude::*;

// manual re-exports
pub use current_input_state::CurrentInputState; // ui::EmptyWidget
pub use crate::widget::EmptyWidget; // ui::EmptyWidget
