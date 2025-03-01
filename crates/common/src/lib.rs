pub mod data;
pub mod math;
pub mod errors;
pub mod instant;
pub mod graphics;


// TODO: nuke this ?!?!?!?1
pub trait Dropdownable2: Send + Sync {
    type T:std::fmt::Display + Sized + Send + Sync;
    fn variants() -> Vec<Self::T>;
}


pub mod prelude {
    pub use crate::data::*;
    pub use crate::math::*;
    pub use crate::errors::*;
    pub use crate::instant::*;
    pub use crate::graphics::*;
    pub use crate::Dropdownable2;

    pub(crate) mod lua {
        pub use mlua::Value as LuaValue;
        pub use mlua::FromLua;
        pub use mlua::Error::FromLuaConversionError;
        pub use mlua::prelude::LuaResult;
        pub use mlua::Lua;
    }
}
