use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct SyValueHelper {
    key: String,
    value: CustomElementValue,
}
impl SyValueHelper {
    pub fn new(key: impl ToString) -> Self {
        Self {
            key: key.to_string(), 
            value: CustomElementValue::None,
        }
    }

    pub fn check(&mut self, values: &ValueCollection) -> bool {
        let Ok(value) = values.get_raw(&self.key) else { return false };
        if value == &self.value { return false }

        self.value = value.clone();
        true
    }
}

impl Deref for SyValueHelper {
    type Target = CustomElementValue;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}