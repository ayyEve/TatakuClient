use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct SYValueHelper {
    key: String,
    value: CustomElementValue,
}
impl SYValueHelper {
    pub fn new(key: impl ToString, value: impl Into<CustomElementValue>) -> Self {
        Self {
            key: key.to_string(), 
            value: value.into(),
        }
    }

    pub fn check(&mut self, values: &ValueCollection) -> bool {
        let Ok(value) = values.get_raw(&self.key) else { return false };
        if value == &self.value { return false }

        self.value = value.clone();
        true
    }
}

impl Deref for SYValueHelper {
    type Target = CustomElementValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}