

pub trait Take: Sized {
    fn take(&mut self) -> Self;
}

impl<D:Default> Take for D {
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}