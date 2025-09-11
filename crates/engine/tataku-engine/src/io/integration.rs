use crate::*;

// TODO: replace check_enabled with a set_enabled, and in settings have the integrations in a HashMap<integration-name, enabled>
pub trait TatakuIntegration: Send + Sync {
    fn name(&self) -> CowStr;

    /// initialize the integration
    fn init(
        &mut self, 
        #[cfg(feature="graphics")]
        _window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> tataku::TatakuResult<()> { Ok(()) }

    /// handle if the integration should be enabled or disabled
    /// 
    /// the integration itself should handle if its enabled or disabled
    /// 
    /// TODO: rename this?
    fn check_enabled(
        &mut self, 
        settings: &engine::Settings
    ) -> tataku::TatakuResult<()>;

    /// handle a tataku event 
    fn handle_event(
        &mut self, 
        _event: &engine::integration_event::TatakuIntegrationEvent, 
        _values: &dyn common::reflect::Reflect,
        _actions: &mut actions::action::ActionQueue,
    ) {}

    /// update the integration
    fn update(
        &mut self, 
        _values: &mut dyn common::reflect::Reflect, 
        _actions: &mut actions::action::ActionQueue,
    ) {}
}


#[derive(Copy, Clone)]
pub struct TatakuIntegrationBuilder {
    pub name: &'static str,
    pub build: fn() -> tataku::TatakuResult<Box<dyn TatakuIntegration>>,
}
