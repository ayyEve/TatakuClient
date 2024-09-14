mod menu;
mod dialog;
mod empty_menu;

pub use menu::*;
pub use dialog::*;
pub use empty_menu::*;


#[derive(Clone, Debug)]
pub enum MenuType {
    Internal(&'static str),
    Custom(String)
}
#[cfg(feature="graphics")]
impl MenuType {
    pub fn from_menu(menu: &dyn AsyncMenu) -> Self {
        let Some(custom) = menu.get_custom_name() else { return Self::Internal(menu.get_name()) };
        Self::Custom(custom.clone())
    }
}
