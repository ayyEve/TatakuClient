use crate::prelude::*;

#[derive(Debug, Eq, PartialEq)]
pub enum TatakuValueError<'a> {
    EntryDoesntExist {
        entry: Cow<'a, str>
    },

    ValueWrongType {
        expected: Cow<'a, str>,
        received: Cow<'a, str>
    }
}
impl<'a> TatakuValueError<'a> {
    pub fn wrong_type(expected: impl Into<Cow<'a, str>>, received: impl Into<Cow<'a, str>>) -> Self {
        Self::ValueWrongType { expected: expected.into(), received: received.into() }
    }
}