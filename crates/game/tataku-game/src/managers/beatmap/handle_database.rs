
/// FIXME: this is a bad name for this
#[derive(Copy, Clone)]
pub enum HandleDatabase {
    No,
    Yes,
    YesAndReturnNewMaps
}
impl HandleDatabase {
    pub(crate) fn insert_into_database(&self) -> bool {
        matches!(self, Self::Yes | Self::YesAndReturnNewMaps)
    }
    pub(crate) fn return_new_maps(&self) -> bool {
        matches!(self, Self::No | Self::YesAndReturnNewMaps)
    }
}
impl From<bool> for HandleDatabase {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}
