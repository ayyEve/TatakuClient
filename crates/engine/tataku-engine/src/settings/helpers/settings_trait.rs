#[cfg(feature="graphics")]
pub trait MakeSettingsMenu {
    fn create_provider(
        &self, 
        prefix: String,
        builder: &mut crate::settings::SettingsBuilder,
    );
}
