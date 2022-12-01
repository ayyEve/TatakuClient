use crate::prelude::*;

// settings helper
pub type SettingsHelper = GlobalObjectValue<Settings>;

// window size helper
pub type WindowSizeHelper = GlobalObjectValue<WindowSize>;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct WindowSize(pub Vector2);
impl WindowSize {
    pub fn get() -> Arc<WindowSize> {
        GlobalObjectManager::get().unwrap_or_default()
    }
}
impl std::ops::Deref for WindowSize {
    type Target = Vector2;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
