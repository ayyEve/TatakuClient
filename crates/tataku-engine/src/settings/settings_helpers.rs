use crate::prelude::*;

// window size helper
pub type WindowSizeHelper = GlobalValue<WindowSize>;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct WindowSize(pub Vector2);
impl WindowSize {
    pub fn get() -> Arc<WindowSize> {
        GlobalValueManager::get().unwrap_or_default()
    }
}
impl std::ops::Deref for WindowSize {
    type Target = Vector2;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
