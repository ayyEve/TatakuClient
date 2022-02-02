
mod dialog;
mod main_menu;
mod pause_menu;
mod score_menu;
mod direct_menu;
mod loading_menu;
mod settings_menu;
mod beatmap_select;


pub use dialog::*;
pub use main_menu::*;
pub use pause_menu::*;
pub use score_menu::*;
pub use direct_menu::*;
pub use loading_menu::*;
pub use settings_menu::*;
pub use beatmap_select::*;
pub use ayyeve_piston_ui::menu::{menu::Menu, menu_elements::*};


use crate::prelude::*;
pub trait ControllerInputMenu<G>:Menu<G> {
    fn controller_down(&mut self, _g:&mut Game, _controller: &Box<dyn Controller>, _button: u8) -> bool {false}
    fn controller_up(&mut self, _g:&mut Game, _controller: &Box<dyn Controller>, _button: u8) -> bool {false}
}