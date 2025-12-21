use crate::*;
use tataku::Color;
use std::sync::atomic::{ AtomicUsize, Ordering };
use engine::game::notifications::NotificationOnClick;

static ID_COUNTER:AtomicUsize = AtomicUsize::new(0);

#[derive(ChainableInitializer)]
#[derive(Clone, Debug, Default2)]
pub struct Notification {
    /// id number for this notification
    #[default(ID_COUNTER.fetch_add(1, Ordering::AcqRel))]
    pub id: usize,

    /// text to display
    #[chain] pub text: String,

    /// color of the bounding box
    #[chain] pub color: Color,

    /// how long this message should last, in ms
    #[chain] pub duration: f32,

    /// what shold happen on click?
    #[chain] pub onclick: game::notifications::NotificationOnClick
}
impl Notification {
    pub fn new(
        text: String, 
        color: Color, 
        duration: f32, 
        onclick: NotificationOnClick
    ) -> Self {
        let id = ID_COUNTER.fetch_add(1, Ordering::AcqRel);
        Self {
            id,
            text,
            color,
            duration,
            onclick
        }
    }
    pub fn new_text(
        text: impl Into<String>, 
        color: Color, 
        duration: f32
    ) -> Self {
        Self::new(
            text.into(), 
            color, 
            duration, 
            NotificationOnClick::None
        )
    }

    pub fn new_error(
        text: impl Into<String>, 
        err: impl Into<tataku::Error>
    ) -> Self {
        Self::new(
            format!("{}\n{:?}", text.into(), err.into()),
            Color::RED,
            5_000.0, 
            NotificationOnClick::None,
        )
    }
}

// impl Default for Notification {
//     fn default() -> Self {
//         let id = ID_COUNTER.fetch_add(1, Ordering::AcqRel);
//         Self {
//             id,
//             text: String::new(),
//             color: Color::WHITE,
//             duration: 0.0,
//             onclick: NotificationOnClick::None,
//         }
//     }
// }
