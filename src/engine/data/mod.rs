mod pool;
mod take;
mod shunting_yard;
mod sy_change_check;
mod tataku_variables;
mod shunting_yard_intos;

pub use pool::*;
pub use take::*;
pub use shunting_yard::*;
pub use sy_change_check::*;
pub use tataku_variables::*;


pub trait Nope {
    fn nope(self);
}
impl<T> Nope for T {
    fn nope(self) {}
}