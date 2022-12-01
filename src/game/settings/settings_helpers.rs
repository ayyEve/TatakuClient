use crate::prelude::*;
use std::ops::{ Deref, DerefMut };
use tokio::sync::RwLockWriteGuard;

/// helper so when a mutable reference to settings is dropped, it sends out an update with the new info
pub struct MutSettingsHelper<'a> {
    guard: RwLockWriteGuard<'a, Settings>
}
impl<'a> MutSettingsHelper<'a> {
    pub fn new(guard:RwLockWriteGuard<'a, Settings>) -> Self {
        Self {
            guard
        }
    }
}
impl<'a> Deref for MutSettingsHelper<'a> {
    type Target = RwLockWriteGuard<'a, Settings>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}
impl<'a> DerefMut for MutSettingsHelper<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}
impl<'a> Drop for MutSettingsHelper<'a> {
    fn drop(&mut self) {
        trace!("mut settings dropped");
        // assume something was changed for now
        let a = Arc::new(self.guard.clone());
        GlobalObjectManager::update(a)
    }
}


// settings helper
pub type SettingsHelper = GlobalObjectInstance<Settings>;

// window size helper
pub type WindowSizeHelper = GlobalObjectInstance<WindowSize>;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct WindowSize(pub Vector2);
impl WindowSize {
    pub fn get() -> Arc<WindowSize> {
        GlobalObjectManager::get().unwrap_or_default()
    }
}
impl Deref for WindowSize {
    type Target = Vector2;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}