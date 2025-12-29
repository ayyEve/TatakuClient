pub mod cursor;
pub mod display;
pub mod logging;
pub mod helpers;
pub mod settings;
pub mod connection;
pub mod integration;
pub mod common_gameplay;
pub mod background_game;

pub use self::helpers::*;
pub use settings::Settings;


use crate::*;

#[derive(Default)]
#[cfg(feature="graphics")]
pub struct SettingsCategory {
    pub name: String,
    pub properties: Vec<Box<dyn ui::widget::Widget<actions::Action>>>, 
    pub values: Vec<Box<dyn ui::widget::Widget<actions::Action>>>,
    pub names: Vec<String>,
}
