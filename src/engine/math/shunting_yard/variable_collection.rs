use crate::prelude::*;

#[derive(Default, Debug)]
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

    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }

    pub fn exists(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn get_raw(&self, key: &str) -> Result<&CustomElementValue, ShuntingYardError> {
        // TODO: handle hashmap entries
        if let Some(v) = self.0.get(key) {
            return Ok(v)
        }

        //TODO: optimize this, this is quite bad
        let mut remaining = key.split(".").collect::<Vec<_>>();
        // let mut entries = Vec::new();

        if remaining.len() > 1 {
            let k2 = remaining.pop().unwrap();
            let key = remaining.join(".");

            if let Some(CustomElementValue::Map(m)) = self.0.get(&key) {
                if let Some(v) = m.get(k2) {
                    return Ok(v);
                }
            }
        }

        // while remaining.len() > 0 {
        //     entries.push(remaining.pop().unwrap());
        //     let key = remaining.join(".");
        //     info!("checking key: {key}");
        //     if let Some(CustomElementValue::Map(m)) = self.0.get(&key) {
        //         let key2 = entries.join(".");
        //         info!("checking key2: {}");
        //         if let Some(v) = m.get(&key) {
        //             return Ok(v)
        //         }
        //     }
        // }

        Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
    }
    pub fn get_f32(&self, key: &str) -> Result<f32, ShuntingYardError> {
        match self.get_raw(key) {
            Ok(CustomElementValue::String(_)) => Err(ShuntingYardError::ValueIsntANumber(key.to_owned())),
            Ok(other) => other.as_f32(),
            Err(_) => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }
    }
    pub fn get_string(&self, key: &str) -> Result<String, ShuntingYardError> {
        self
            .get_raw(key)
            .map(|i| i.as_string())
            // .ok_or_else(|| ShuntingYardError::EntryDoesntExist(key.to_owned()))
    }

    pub fn get_bool<'a>(&self, key: &str) -> Result<bool, ShuntingYardError> {
        match self.get_raw(key) {
            Ok(CustomElementValue::Bool(b)) => Ok(*b),
            Ok(_) => Err(ShuntingYardError::ValueIsntABool),
            _ => Err(ShuntingYardError::EntryDoesntExist(key.to_owned()))
        }
    }
}
