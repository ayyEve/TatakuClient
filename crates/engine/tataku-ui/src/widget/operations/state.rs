use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum StateOperation {
    Add(ElementState),
    Remove(ElementState),
    Toggle(ElementState),
}
