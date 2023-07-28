use crate::prelude::*;

static ID_COUNTER:AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct Notification {
    /// id number for this notification
    pub id: usize,
    /// text to display
    pub text: String,
    /// color of the bounding box
    pub color: Color,
    /// how long this message should last, in ms
    pub duration: f32,
    /// what shold happen on click?
    pub onclick: NotificationOnClick
}
impl Notification {
    pub fn new(text: String, color: Color, duration: f32, onclick: NotificationOnClick) -> Self {
        let id = ID_COUNTER.fetch_add(1, Ordering::AcqRel);
        Self {
            id,
            text,
            color,
            duration,
            onclick
        }
    }
}
