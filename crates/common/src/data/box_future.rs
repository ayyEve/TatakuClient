/// stolen from futures-util. i use this in several crates, but nothing else from futures util
pub type BoxFuture<'a, T> = std::pin::Pin<std::boxed::Box<dyn Future<Output = T> + Send + 'a>>;