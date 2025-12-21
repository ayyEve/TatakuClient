use tataku_engine::*;
use std::fmt::Debug;
use serde_binary::binary_stream::Endian;

const ENDIAN: Endian = Endian::Little;

#[derive(Clone, Default, Debug)]
pub(crate) struct Wrapper<T>(T);
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Wrapper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        T::deserialize(deserializer).map(Self)
    }
}
impl<T: serde::Serialize> serde::Serialize for Wrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.0.serialize(serializer)
    }
}

impl<T> redb::Value for Wrapper<T> 
where T:
    serde::Serialize + serde::de::DeserializeOwned 
    + std::fmt::Debug
    + Default
    + 'static 
{
    type SelfType<'a> = T;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> { None }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a {
        serde_binary::from_slice::<T>(data, ENDIAN).unwrap_or_default()
    }

    fn as_bytes<'a, 'b: 'a>(
        value: &'a Self::SelfType<'b>
    ) -> Self::AsBytes<'a> where Self: 'b {
        serde_binary::to_vec(value, ENDIAN).unwrap()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new(&format!("tataku-{}", std::any::type_name::<T>()))
    }
}
