use crate::prelude::*;

#[async_trait]
pub trait GameModeInput: Send + Sync {
    async fn key_down(&mut self, key:Key) -> Option<ReplayAction>;
    async fn key_up(&mut self, key:Key) -> Option<ReplayAction>;
    async fn on_text(&mut self, _text: &String, _mods: &KeyModifiers) -> Option<ReplayAction> { None }


    async fn mouse_move(&mut self, _pos:Vector2) -> Option<ReplayAction> { None }
    async fn mouse_down(&mut self, _btn:MouseButton) -> Option<ReplayAction> { None }
    async fn mouse_up(&mut self, _btn:MouseButton) -> Option<ReplayAction> { None }
    async fn mouse_scroll(&mut self, _delta:f32) -> Option<ReplayAction> { None }


    async fn controller_press(&mut self, _c: &GamepadInfo, _btn: ControllerButton) -> Option<ReplayAction> { None }
    async fn controller_release(&mut self, _c: &GamepadInfo, _btn: ControllerButton) -> Option<ReplayAction> { None }
    async fn controller_axis(&mut self, _c: &GamepadInfo, _axis_data:HashMap<Axis, (bool, f32)>) -> Option<ReplayAction> { None }
}

