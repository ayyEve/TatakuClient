use serde::de::{Deserialize, Deserializer};

#[derive(Debug, Default)]
pub enum TatakuSettingOptional<T> {
    #[default]
    NoValue,
    Err(String),
    Value(T)
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for TatakuSettingOptional<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match T::deserialize(deserializer) {
            Ok(t) => Ok(TatakuSettingOptional::Value(t)),
            Err(e) => Ok(TatakuSettingOptional::Err(e.to_string())),
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod test2 {
    use crate::prelude::*;
    use super::*;
    
    #[derive(Deserialize, Debug)]
    struct Test {
        a: TatakuSettingOptional<String>,
        b: TatakuSettingOptional<i32>,
    }


    #[derive(Debug, Serialize)]
    #[derive(SettingsDeserialize)]
    #[serde(from="Test2Deserializer")]
    struct Test2 {
        a: String,
        b: i32,
    }
    impl Default for Test2 {
        fn default() -> Self {
            Self {
                a: "default".to_owned(),
                b: 100
            }
        }
    }

    #[test]
    fn test() {
        let test = "{\"a\":\"Text\", \"b\":100}";
        let a: Test = serde_json::from_str(test).expect("nope");
        println!("1: {a:?}");


        let test2 = "{\"a\":\"Text\", \"b\":\"not a number\"}";
        let a: Test = serde_json::from_str(test2).expect("nope");
        println!("2: {a:?}");
    }

    #[test]
    fn test2() {
        // good
        let test = "{\"a\":\"Text\", \"b\":100}";
        let a: Test2 = serde_json::from_str(test).expect("nope");
        println!("1: {a:?}");

        // invalid b
        let test2 = "{\"a\":\"Text\", \"b\":\"not a number\"}";
        let a: Test2 = serde_json::from_str(test2).expect("nope");
        println!("2: {a:?}");

        // missing b
        let test3 = "{\"a\":\"Text\"}";
        let a: Test2 = serde_json::from_str(test2).expect("nope");
        println!("3: {a:?}");
    }

}
