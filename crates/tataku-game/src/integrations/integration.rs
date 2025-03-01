use crate::prelude::*;



// TODO: replace check_enabled with a set_enabled, and in settings have the integrations in a HashMap<integration-name, enabled>
pub trait TatakuIntegration: Send + Sync {
    fn name(&self) -> Cow<'static, str>;

    /// initialize the integration
    fn init(
        &mut self, 
        settings: &Settings
    ) -> TatakuResult<()>;

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
    fn handle_event(&mut self, _event: &TatakuIntegrationEvent, _values: &ValueCollection) {}

    /// update the integration
    fn update(
        &mut self, 
        _values: &mut ValueCollection, 
        _actions: &mut ActionQueue
    ) {}
}
