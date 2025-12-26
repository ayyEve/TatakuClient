pub mod optional;
pub mod color;
pub mod gamemode_collection;

#[cfg(feature="ui")] pub mod builder;
#[cfg(feature="ui")] pub mod provider;

pub use optional::*;
pub use color::*;
pub use gamemode_collection::*;


#[cfg(feature="ui")] pub use builder::*;
#[cfg(feature="ui")] pub use provider::*;

#[cfg(feature="graphics")]
pub trait MakeSettingsMenu {
    fn create_provider(
        &self,
        prefix: String,
        builder: &mut crate::settings::SettingsBuilder,
    );
}
