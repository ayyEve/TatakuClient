use crate::*;

pub struct AudioApiInit {
    pub name: &'static str,
    pub init: fn() -> tataku::TatakuResult<Arc<dyn AudioApi>>,
}
