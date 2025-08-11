use std::sync::Arc;

pub fn serialize<S: serde::Serializer>(t: &Arc<str>, s: S) -> Result<S::Ok, S::Error> {
    <&str as serde::ser::Serialize>::serialize(&&**t, s)
}

pub fn deserialize<'de, D>(a: D) -> Result<Arc<str>, D::Error> where D: serde::Deserializer<'de> {
    let a = <String as serde::de::Deserialize>::deserialize(a)?;
    Ok(a.into())
}