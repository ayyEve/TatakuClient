pub mod data;
pub mod math;
pub mod errors;
pub mod prelude;
pub mod instant;
pub mod graphics;


// TODO: nuke this ?!?!?!?1
pub trait Dropdownable2 {
    type T:std::fmt::Display+Sized;
    fn variants() -> Vec<Self::T>;
}
