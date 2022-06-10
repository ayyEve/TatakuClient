use crate::prelude::*;
use std::ops::{ Deref, DerefMut };
use tokio::sync::RwLockWriteGuard;

lazy_static::lazy_static! {
    static ref SETTINGS_CHECK: (Arc<parking_lot::Mutex<MultiFuze<Arc<Settings>>>>, MultiBomb<Arc<Settings>>) = {
        let (f, b) = MultiBomb::new();
        (Arc::new(parking_lot::Mutex::new(f)), b)
    };
}


pub struct SettingsHelper {
    /// cached settings
    settings: Arc<Settings>,

    /// what checks for new settings updates
    settings_bomb: MultiBomb<Arc<Settings>>,
}
impl SettingsHelper {
    pub async fn new() -> Self {
        let settings = get_settings!();
        Self {
            settings: Arc::new(settings.clone()),
            settings_bomb: SETTINGS_CHECK.1.clone(),
        }
    }

    pub fn update(&mut self) -> bool {
        let mut changed = false;
        // while to get the most up-to-date settings
        while let Some(settings) = self.settings_bomb.exploded() {
            self.settings = settings;
            changed |= true;
        }
        changed
    }
}
impl Deref for SettingsHelper {
    type Target = Arc<Settings>;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl Default for SettingsHelper {
    fn default() -> Self {
        Self { 
            settings: Arc::new(Settings::default()), 
            settings_bomb: SETTINGS_CHECK.1.clone() 
        }
    }
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

impl<'a> std::ops::Deref for MutSettingsHelper<'a> {
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