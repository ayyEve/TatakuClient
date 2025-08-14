pub trait ChainMaybe: Sized {
    fn chain_bool(self, check: bool, f: impl Fn(Self) -> Self) -> Self;
    fn chain_maybe<T>(self, op: Option<T>, f: impl Fn(Self, T) -> Self) -> Self;
}
impl<S> ChainMaybe for S {
    fn chain_bool(self, check: bool, f: impl Fn(Self) -> Self) -> Self {
        if !check { return self }
        f(self)
    }
    fn chain_maybe<T>(self, op: Option<T>, f: impl Fn(Self, T) -> Self) -> Self {
        let Some(op) = op else { return self };
        f(self, op)
    }
}
