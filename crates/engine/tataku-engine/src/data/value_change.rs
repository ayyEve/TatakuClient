use crate::*;
use common::reflect::*;

// TODO: rename this
#[derive(Clone, Debug)]
pub struct ValueChange<T: Reflect + Clone + PartialEq> {
    key: String,
    value: Option<T>,
}
impl<T: Reflect + Clone + PartialEq> ValueChange<T> {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
        }
    }

    pub fn update(&mut self, values: &dyn Reflect) -> Result<Option<&T>, ReflectError<'_>> {
        let value = values.reflect_get::<T>(&self.key)?;
        let value = &*value;
        if Some(value) == self.value.as_ref() { return Ok(None) }

        self.value = Some(value.clone());
        Ok(self.value.as_ref())
    }

    pub fn try_get(&self) -> tataku::Result<&T> {
        Ok(self.value.as_ref().ok_or(ReflectError::entry_not_exist(&self.key))?)
    }
}

impl<T: Reflect + Clone + PartialEq> Deref for ValueChange<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
