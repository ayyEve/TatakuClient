use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum CursorAction {
    /// Set if the cursor is visible or not
    SetVisible(bool),
    /// Set/remove an override for the Cursor's ripple radius
    OverrideRippleRadius(Option<f32>),
}
impl From<CursorAction> for TatakuAction {
    fn from(value: CursorAction) -> Self {
        Self::CursorAction(value)
    }
}


#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CursorMode {
    /// regular cursor image
    Normal,
    HorizontalResize,
    VerticalResize,
    
    /// Both horizontal and vertical resize
    Resize,

    /// Hand pointing at thing
    Pointer,

    /// text cursor
    Text,
}
impl CursorMode {
    pub fn tex_name(&self) -> &str {
        match self {
            CursorMode::Normal => "tataku_cursor_normal",
            CursorMode::HorizontalResize => "tataku_cursor_hresize",
            CursorMode::VerticalResize => "tataku_cursor_vresize",
            CursorMode::Resize => "tataku_cursor_resize",
            CursorMode::Pointer => "tataku_cursor_pointer",
            CursorMode::Text => "tataku_cursor_text",
        }
    }
}
