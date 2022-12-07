use crate::prelude::*;

#[async_trait]
pub trait GameModeInput: Send + Sync {
    async fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager);
    async fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager);
    async fn on_text(&mut self, _text: &String, _mods: &KeyModifiers, _manager: &mut IngameManager) {}


    async fn mouse_move(&mut self, _pos:Vector2, _manager:&mut IngameManager) {}
    async fn mouse_down(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    async fn mouse_up(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    async fn mouse_scroll(&mut self, _delta:f64, _manager:&mut IngameManager) {}


    async fn controller_press(&mut self, _c: &Box<dyn Controller>, _btn: u8, _manager:&mut IngameManager) {}
    async fn controller_release(&mut self, _c: &Box<dyn Controller>, _btn: u8, _manager:&mut IngameManager) {}
    async fn controller_hat_press(&mut self, _hat: piston::controller::ControllerHat, _manager:&mut IngameManager) {}
    async fn controller_hat_release(&mut self, _hat: piston::controller::ControllerHat, _manager:&mut IngameManager) {}
    async fn controller_axis(&mut self, _c: &Box<dyn Controller>, _axis_data:HashMap<u8, (bool, f64)>, _manager:&mut IngameManager) {}
}

