mod game;
mod tasks;
mod values;
mod managers;
mod database;


pub mod prelude {
    pub use crate::game::*;
    pub use crate::tasks::*;
    pub use crate::values::*;
    pub use crate::managers::*;
    pub use crate::database::*;
    
    pub use tataku_engine::prelude::*;
    #[cfg(feature="graphics")] pub use tataku_ui::prelude::*;
    #[cfg(feature="graphics")] pub use tataku_graphics::prelude::*;
    #[cfg(feature="graphics")] pub use tataku_interface::prelude::*;
}
