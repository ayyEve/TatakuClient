

pub trait Take: Sized {
    /// shorthand for std::mem::take
    fn take(&mut self) -> Self;
}

impl<D:Default> Take for D {
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}