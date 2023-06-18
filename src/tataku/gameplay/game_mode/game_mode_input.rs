use crate::prelude::*;

#[async_trait]
pub trait GameModeInput: Send + Sync {
    async fn key_down(&mut self, key:Key) -> Option<ReplayFrame>;
    async fn key_up(&mut self, key:Key) -> Option<ReplayFrame>;
    async fn on_text(&mut self, _text: &String, _mods: &KeyModifiers) -> Option<ReplayFrame> { None }


    async fn mouse_move(&mut self, _pos:Vector2) -> Option<ReplayFrame> { None }
    async fn mouse_down(&mut self, _btn:MouseButton) -> Option<ReplayFrame> { None }
    async fn mouse_up(&mut self, _btn:MouseButton) -> Option<ReplayFrame> { None }
    async fn mouse_scroll(&mut self, _delta:f32) -> Option<ReplayFrame> { None }


    async fn controller_press(&mut self, _c: &GamepadInfo, _btn: ControllerButton) -> Option<ReplayFrame> { None }
    async fn controller_release(&mut self, _c: &GamepadInfo, _btn: ControllerButton) -> Option<ReplayFrame> { None }
    async fn controller_axis(&mut self, _c: &GamepadInfo, _axis_data:HashMap<Axis, (bool, f32)>) -> Option<ReplayFrame> { None }
}

