mod tree;
mod style;
mod widget;
mod message;
mod tataku_event;
mod spatial_navigation;
mod current_input_state;

pub mod prelude {
    pub(crate) use std::str::FromStr;
    pub(crate) use tataku_common::prelude::*;
    pub(crate) use tataku_graphics::prelude::*;
    pub(crate) use tataku_client_common::prelude::*;
    pub(crate) use tataku_client_common::prelude::TatakuValue;

    pub use crate::tree::*;
    pub use crate::style::*;
    pub use crate::widget::*;
    pub use crate::message::*;
    pub use crate::tataku_event::*;
    pub use crate::spatial_navigation::*;
    pub use crate::current_input_state::*;

    pub const EMPTY_NODE: NodeId = NodeId {
        node_id: taffy::NodeId::new(u64::MAX),
        owner: MessageOwner::Menu,
    };
    pub const FILL: CssUnit = CssUnit::Percent(f16::from_f32_const(1.0));
    pub const SHRINK: CssUnit = CssUnit::Auto;
}