use crate::*;

pub struct AudioApiInit {
    pub name: &'static str,
    pub init: fn() -> tataku::Result<Arc<dyn AudioApi>>,
}
