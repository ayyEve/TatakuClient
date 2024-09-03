use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct SyValueHelper<T: Reflect + Clone + PartialEq> {
    key: String,
    value: Option<T>,
}
impl<T: Reflect + Clone + PartialEq> SyValueHelper<T> {
    pub fn new(key: impl ToString) -> Self {
        Self {
            key: key.to_string(),
            value: None,
        }
    }

    pub fn update(&mut self, values: &ValueCollection) -> Result<Option<&T>, ReflectError<'_>> {
        let value = values.as_dyn().reflect_get::<T>(&self.key)?;
        if Some(value) == self.value.as_ref() { return Ok(None) }

        self.value = Some(value.clone());
        Ok(self.value.as_ref())
    }

    pub fn try_get(&self) -> TatakuResult<&T> {
        Ok(self.value.as_ref().ok_or(ReflectError::entry_not_exist(&self.key))?)
    }
}

impl<T: Reflect + Clone + PartialEq> Deref for SyValueHelper<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
