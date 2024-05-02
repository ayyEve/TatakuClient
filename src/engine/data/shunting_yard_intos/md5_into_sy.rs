use crate::prelude::*;

impl From<Md5Hash> for TatakuValue {
    fn from(hash: Md5Hash) -> Self {
        Self::String(hash.to_string())
    }
}
impl TryInto<Md5Hash> for &TatakuValue {
    type Error = String;
    fn try_into(self) -> Result<Md5Hash, Self::Error> {
        let str = self.as_string();
        Md5Hash::try_from(str).map_err(|e| format!("{e:?}"))
    }
}