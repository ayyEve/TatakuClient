use crate::prelude::*;

#[derive(Default)]
pub struct ShuntingYardValues(HashMap<String, CustomElementValue>);
impl ShuntingYardValues {
    pub fn set_chained(mut self, key: impl ToString, value: impl Into<CustomElementValue>) -> Self {
        self.set(key, value);
        self
    }

    pub fn set(&mut self, key: impl ToString, value: impl Into<CustomElementValue>) {
        let value = value.into();
        let key = key.to_string();
        self.0.insert(key, value);
    }

    pub fn get_f32(&self, key: &String) -> Result<f32, ShuntingYardError> {
        match self.0.get(key) {
            Some(CustomElementValue::String(_)) => Err(ShuntingYardError::ValueIsntANumber(key.clone())),
            Some(other) => other.as_f32(),
            None => Err(ShuntingYardError::EntryDoesntExist(key.clone()))
        }
    }
    pub fn get_string(&self, key: &String) -> Result<String, ShuntingYardError> {
        self.0
            .get(key)
            .map(|i| i.as_string())
            .ok_or_else(|| ShuntingYardError::EntryDoesntExist(key.clone()))
    }
}