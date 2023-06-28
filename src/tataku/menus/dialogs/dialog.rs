use crate::prelude::*;


/// a dialog is basically just a menu, except it does not occupy a whole gamemode,
/// and should be drawn overtop every other menu
#[async_trait]
pub trait Dialog<G:Send+Sync>:Send+Sync {
    fn get_bounds(&self) -> Rectangle;
    fn should_close(&self) -> bool;
    fn name(&self) -> &'static str {""}

    /// dialog is being forcefully closed
    async fn force_close(&mut self) {}

    async fn update(&mut self, _g:&mut G) {}
    async fn draw(&mut self, list: &mut RenderableCollection);

    // input handlers
    async fn on_mouse_move(&mut self, _pos:Vector2, _g:&mut G) {}
    async fn on_mouse_scroll(&mut self, _delta:f32, _g:&mut G) -> bool {false}
    async fn on_mouse_down(&mut self, _pos:Vector2, _button:MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    async fn on_mouse_up(&mut self, _pos:Vector2, _button:MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}

    async fn on_text(&mut self, _text:&String) -> bool {false}
    async fn on_key_press(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    async fn on_key_release(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>);

    async fn on_controller_press(&mut self, _controller: &GamepadInfo, _button: ControllerButton) -> bool {false}
    async fn on_controller_release(&mut self, _controller: &GamepadInfo, _button: ControllerButton) -> bool {false}
    async fn on_controller_axis(&mut self, _controller: &GamepadInfo, _axis_data: &HashMap<Axis, (bool, f32)>) {}

    fn string_function1(&mut self, _val: String) {}
    // fn string_function2(&mut self, _val: String) {}

    fn draw_background(&mut self, color:Color, list: &mut RenderableCollection) {
        let bounds = self.get_bounds();
        list.push(Rectangle::new(
            bounds.pos,
            bounds.size,
            color.alpha(0.8),
            Some(Border::new(color, 2.0))
        ))
    }


    
}

// // toolbar options
// const TOOLBAR_HEIGHT:f64 = 20.0;

// /// top bar helper, close, move, etc
// pub struct DialogBar {
//     pub move_start: Option<Vector2>
// }
// impl DialogBar {
//     fn update<G, D:Dialog<G>>(&self, dialog: D) {

//     }
// }