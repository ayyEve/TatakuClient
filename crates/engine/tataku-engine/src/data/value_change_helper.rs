use crate::*;
use common::reflect::*;

// TODO: rename this
#[derive(Clone, Debug)]
pub struct ValueChangeHelper<T> {
    key: String,
    value: Option<T>,
}
impl<T> ValueChangeHelper<T> {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
        }
    }
}
impl<T: Reflect + Clone + PartialEq> ValueChangeHelper<T> {
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

impl<T> Deref for ValueChangeHelper<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
