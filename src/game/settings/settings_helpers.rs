use crate::prelude::*;
use std::ops::{ Deref, DerefMut };
use tokio::sync::RwLockWriteGuard;

lazy_static::lazy_static! {
    static ref SETTINGS_CHECK: (Arc<parking_lot::Mutex<MultiFuze<Arc<Settings>>>>, MultiBomb<Arc<Settings>>) = {
        let (f, b) = MultiBomb::new();
        (Arc::new(parking_lot::Mutex::new(f)), b)
    };
    
    static ref WINDOW_SIZE_CHECK: (Arc<parking_lot::Mutex<MultiFuze<Arc<WindowSize>>>>, MultiBomb<Arc<WindowSize>>) = {
        let (f, b) = MultiBomb::new();
        (Arc::new(parking_lot::Mutex::new(f)), b)
    };

    static ref CURRENT_WINDOW_SIZE: Arc<parking_lot::RwLock<Arc<WindowSize>>> = Arc::new(parking_lot::RwLock::new(Arc::new(WindowSize(Vector2::zero()))));
}

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
        info!("mut settings dropped");
        // assume something was changed for now
        let a = Arc::new(self.guard.clone());
        SETTINGS_CHECK.0.lock().ignite(a);
    }
}


// settings helper
pub type SettingsHelper = EventHandler<Settings>;
impl EventHandlerReceiver for Settings {
    fn get_receiver() -> MultiBomb<Arc<Self>> {
        SETTINGS_CHECK.1.clone() 
    }
}
#[async_trait]
impl EventHandlerInitial for Settings {
    async fn get_initial() -> Arc<Self> {
        Arc::new(get_settings!().clone())
    }
}


// window size helper
pub type WindowSizeHelper = EventHandler<WindowSize>;

#[derive(Copy, Clone)]
pub struct WindowSize(pub Vector2);
impl WindowSize {
    pub fn get() -> Arc<WindowSize> {
        CURRENT_WINDOW_SIZE.read().clone()
    }
}


impl Deref for WindowSize {
    type Target = Vector2;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl EventHandlerReceiver for WindowSize {
    fn get_receiver() -> MultiBomb<Arc<Self>> {
        WINDOW_SIZE_CHECK.1.clone() 
    }
}
#[async_trait]
impl EventHandlerInitial for WindowSize {
    async fn get_initial() -> Arc<Self> {
        Self::get()
    }
}
impl Default for WindowSize {
    fn default() -> Self {
        Self(Default::default())
    }
}


pub fn set_window_size(new_size: Vector2) {
    let a = Arc::new(WindowSize(new_size));
    *CURRENT_WINDOW_SIZE.write() = a.clone();
    WINDOW_SIZE_CHECK.0.lock().light(a);
}