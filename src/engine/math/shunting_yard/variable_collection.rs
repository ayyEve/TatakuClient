use crate::prelude::*;

#[derive(Default)]
pub struct ShuntingYardValues(HashMap<String, CustomElementValue>);
impl ShuntingYardValues {
    // initialize with some basic values
    pub fn new() -> Self {
        Self::default()
            .set_chained("true", true)
            .set_chained("false", false)
    }


    pub fn set_chained(mut self, key: impl ToString, value: impl Into<CustomElementValue>) -> Self {
        self.set(key, value);
        self
    }

    pub fn set(&mut self, key: impl ToString, value: impl Into<CustomElementValue>) {
        let value = value.into();
        let key = key.to_string();
        self.0.insert(key, value);
    }
    pub fn set_chained_mut(&mut self, key: impl ToString, value: impl Into<CustomElementValue>) -> &mut Self {
        self.set(key, value);
        self
    }

    pub fn set_multiple(&mut self, list: impl Iterator<Item=(impl ToString, impl Into<CustomElementValue>)>) {
        for (key, value) in list {
            self.set(key, value);
        }
    }

    pub fn get_f32(&self, key: &str) -> Result<f32, ShuntingYardError> {
        match self.0.get(key) {
            Some(CustomElementValue::String(_)) => Err(ShuntingYardError::ValueIsntANumber(key.to_owned())),
            Some(other) => other.as_f32(),
            None => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }
    }
    pub fn get_string(&self, key: &str) -> Result<String, ShuntingYardError> {
        self.0
            .get(key)
            .map(|i| i.as_string())
            .ok_or_else(|| ShuntingYardError::EntryDoesntExist(key.to_owned()))
    }

    pub fn get_bool<'a>(&self, key: &str) -> Result<bool, ShuntingYardError> {
        match self.0.get(key) {
            Some(CustomElementValue::Bool(b)) => Ok(*b),
            Some(_) => Err(ShuntingYardError::ValueIsntABool),
            _ => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }
    }
}