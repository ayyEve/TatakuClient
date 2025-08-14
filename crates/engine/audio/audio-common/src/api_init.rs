use crate::prelude::*;

pub struct AudioApiInit {
    pub name: &'static str,
    pub init: fn() -> TatakuResult<Arc<dyn AudioApi>>,
}
