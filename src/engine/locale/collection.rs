use crate::prelude::*;
use serde_json::Value;
use strfmt::Format;

pub struct LocaleCollection(Value);
impl LocaleCollection {
    pub fn get_key(&self, key: impl AsRef<str>, variables: &HashMap<String, TatakuValue>) -> String {
        let key = key.as_ref();
        let str = self.0.pointer(key).map(|v|v.to_string()).unwrap_or_else(||key.to_owned());
        match str.format(variables) {
            Ok(s) => s,
            Err(e) => e.to_string()
        }
    }
}
