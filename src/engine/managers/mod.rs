mod instant;
mod input_manager;
mod volume_control;
mod cursor_manager;
mod notification_manager;

pub use instant::*;
pub use volume_control::*;
pub use input_manager::*;
pub use cursor_manager::*;
pub use notification_manager::*;



#[macro_export]
macro_rules! create_value_helper {
    ($struct: ident, $type: ty, $helper_name: ident) => {
        #[derive(Default)]
        pub struct $struct(pub $type);
        impl Deref for $struct {
            type Target = $type;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
        
        pub type $helper_name = GlobalValue<$struct>;
    }
}