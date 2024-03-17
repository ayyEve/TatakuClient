use crate::prelude::*;
use rlua::{Value, Error, FromLua};

pub enum MaybeConditional<T> {
    NotConditional(T),
    Conditional {
        condition: ElementCondition,
        if_true: T,
        if_false: Option<T>
    }
}
impl<T> MaybeConditional<T> {
    pub fn build(&mut self) {
        let Self::Conditional { condition, if_true, if_false } = self else { return };
        condition.build();
        // TODO:: need to build if_true and if_false somehow
    }

    pub fn resolve(&self, values: &ValueCollection) -> Option<&T> {
        match self {
            Self::NotConditional(t) => Some(t),
            Self::Conditional { condition, if_true, if_false } => {
                match condition.resolve(values) {
                    ElementResolve::True => Some(if_true),
                    ElementResolve::False => if_false.as_ref(),
                    ElementResolve::Unbuilt(c) => panic!("condition not build: {c}"),
                    ElementResolve::Error(err) => {
                        error!("error with conditional: {err:?}");
                        None
                    }

                    ElementResolve::Failed => None,
                }
            }
        }
    }
}

// impl<'lua, T:FromLua<'lua>> FromLua for MaybeConditional<T> {

// }