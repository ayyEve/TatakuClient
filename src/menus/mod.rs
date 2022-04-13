mod menu;
mod dialogs;
mod main_menu;
mod pause_menu;
mod score_menu;
mod direct_menu;
mod loading_menu;
mod settings_menu;
mod beatmap_select;

pub use menu::*;
pub use dialogs::*;
pub use main_menu::*;
pub use pause_menu::*;
pub use score_menu::*;
pub use direct_menu::*;
pub use loading_menu::*;
pub use settings_menu::*;
pub use beatmap_select::*;
pub use ayyeve_piston_ui::menu::{menu::Menu, menu_elements::*};

#[async_trait]
pub trait ControllerInputMenu<G:Send+Sync>:AsyncMenu<G> + Send + Sync {
    async fn controller_down(&mut self, _g:&mut Game, _controller: &Box<dyn Controller>, _button: u8) -> bool {false}
    async fn controller_up(&mut self, _g:&mut Game, _controller: &Box<dyn Controller>, _button: u8) -> bool {false}
    async fn controller_axis(&mut self, _g:&mut Game, _controller: &Box<dyn Controller>, _axis_data: HashMap<u8, (bool, f64)>) -> bool {false}
}