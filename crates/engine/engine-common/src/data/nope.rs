
/// when you just dont want to deal with an output
/// 
/// mainly used for inline match statements
/// 
/// 
/// ```rust
/// match a {
///     c => no_return_thing(b),
///     b => returns_thing(b), // angry!!
/// }
/// ```
/// 
/// ```rust
/// match a {
///     c => no_return_thing(b),
///     b => returns_thing(b).nope(), // happy :D
/// }
/// ```
pub trait Nope {
    fn nope(self);
}
impl<T> Nope for T {
    fn nope(self) {}
}
