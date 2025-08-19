use crate::prelude::*;

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
    #[chain] pub onclick: NotificationOnClick
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
        text: impl ToString, 
        color: Color, 
        duration: f32
    ) -> Self {
        Self::new(
            text.to_string(), 
            color, 
            duration, 
            NotificationOnClick::None
        )
    }

    pub fn new_error(
        text: impl ToString, 
        err: impl Into<TatakuError>
    ) -> Self {
        Self::new(
            format!("{}\n{:?}", text.to_string(), err.into()),
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
