use crate::prelude::*;

#[cfg(feature="graphics")]
#[allow(clippy::wrong_self_convention)]
pub trait MakeSettingsMenu {
    fn create_provider(
        &self, 
        prefix: String,
        builder: &mut SettingsBuilder,
    );
}
