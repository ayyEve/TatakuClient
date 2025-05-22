use crate::prelude::*;

// TODO: replace check_enabled with a set_enabled, and in settings have the integrations in a HashMap<integration-name, enabled>
pub trait TatakuIntegration: Send + Sync {
    fn name(&self) -> Cow<'static, str>;

    /// initialize the integration
    fn init(
        &mut self, 
        #[cfg(feature="graphics")]
        _window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> TatakuResult<()> { Ok(()) }

    /// handle if the integration should be enabled or disabled
    /// 
    /// the integration itself should handle if its enabled or disabled
    /// 
    /// TODO: rename this?
    fn check_enabled(
        &mut self, 
        settings: &Settings
    ) -> TatakuResult<()>;

    /// handle a tataku event 
    fn handle_event(
        &mut self, 
        _event: &TatakuIntegrationEvent, 
        _values: &dyn Reflect,
        _actions: &mut ActionQueue,
    ) {}

    /// update the integration
    fn update(
        &mut self, 
        _values: &mut dyn Reflect, 
        _actions: &mut ActionQueue,
    ) {}
}


#[derive(Copy, Clone)]
pub struct TatakuIntegrationBuilder {
    pub name: &'static str,
    pub build: fn() -> TatakuResult<Box<dyn TatakuIntegration>>,
}
