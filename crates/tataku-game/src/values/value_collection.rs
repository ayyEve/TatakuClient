use crate::prelude::*;

#[derive(Default)]
pub struct ValueCollection {
    pub values: TatakuValues,
    pub custom: DynMap,
}
impl Deref for ValueCollection {
    type Target = TatakuValues;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl DerefMut for ValueCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

// TODO: forward the error from values and not the dynmap (or both?)
impl Reflect for ValueCollection {
    fn impl_get<'v, 's>(
        &'s self, 
        path: ReflectPath<'v>
    ) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        self
            .values
            .impl_get(path.clone())
            .or_else(|_| self.custom.impl_get(path))
    }

    fn impl_get_mut<'v>(
        &mut self, 
        path: ReflectPath<'v>
    ) -> ReflectResult<'v, &mut dyn Reflect> {
        self.values
            .impl_get_mut(path.clone())
            .or_else(|_| self.custom.impl_get_mut(path))
    }

    fn impl_insert<'v>(
        &mut self, 
        path: ReflectPath<'v>, 
        value: Box<dyn Reflect>
    ) -> ReflectResult<'v, ()> {
        if self.values.impl_get(path.clone()).is_ok() {
            self.values.impl_insert(path, value)
        } else {
            self.custom.impl_insert(path, value)
        }
    }

    fn impl_iter<'v>(
        &self, 
        path: ReflectPath<'v>
    ) -> ReflectResult<'v, ReflectIter<'_>> {
        match (self.values.impl_iter(path.clone()), self.custom.impl_iter(path)) {
            (Ok(v), Ok(c)) => Ok(ReflectIter::new(
                v.chain(c)
            )),
            (Ok(v), Err(_)) => Ok(v),
            (Err(_), Ok(c)) => Ok(c),
            (Err(ReflectError::EntryNotExist { .. }), Err(e)) => Err(e),
            (Err(e), Err(ReflectError::EntryNotExist { .. })) => Err(e),
            // TODO: is this correct?
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn impl_iter_mut<'v>(
        &mut self, 
        path: ReflectPath<'v>
    ) -> ReflectResult<'v, ReflectIterMut<'_>> {
        match (self.values.impl_iter_mut(path.clone()), self.custom.impl_iter_mut(path)) {
            (Ok(v), Ok(c)) => Ok(ReflectIterMut::new(v.chain(c))),
            (Ok(v), Err(_)) => Ok(v),
            (Err(_), Ok(c)) => Ok(c),
            (Err(ReflectError::EntryNotExist { .. }), Err(e)) => Err(e),
            (Err(e), Err(ReflectError::EntryNotExist { .. })) => Err(e),
            // TODO: is this correct?
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> { None }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}
