use crate::prelude::*;

pub type ActionQueue = Queue<TatakuAction>;

#[derive(Default, Debug)]
pub struct Queue<T>(Vec<T>);
impl<T> Queue<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take(&mut self) -> Vec<T> {
        self.0.take()
    }

    pub fn push(&mut self, item: impl Into<T>) {
        self.0.push(item.into());
    }
    pub fn extend(&mut self, list: impl Into<Vec<T>>) {
        self.0.extend(list.into());
    }
}
impl<T> From<Vec<T>> for Queue<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}
