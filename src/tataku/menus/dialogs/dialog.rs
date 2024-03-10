use crate::prelude::*;


/// a dialog is basically just a menu, except it does not occupy a whole gamemode,
/// and should be drawn overtop every other menu
#[async_trait]
pub trait Dialog:Send+Sync {
    /// internal id name of this dialog
    fn name(&self) -> &'static str {""}
    /// title to display when draggable
    fn title(&self) -> &'static str {""}
    fn get_num(&self) -> usize;
    fn set_num(&mut self, num: usize);

    fn should_close(&self) -> bool;
    // fn get_bounds(&self) -> Bounds;

    fn is_draggable(&self) -> bool { false }

    /// if in a draggable window, is this dialog resizable?
    fn resizable(&self) -> bool { false }

    /// dialog is being forcefully closed
    async fn force_close(&mut self) {}

    fn view(&self) -> IcedElement;
    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues);
    async fn update(&mut self) -> Vec<MenuAction> { Vec::new() }


    // /// if in a draggable dialog, and it was resized
    // async fn resized(&mut self, _new_size: Vector2) {}
    // async fn window_size_changed(&mut self, _window_size: Arc<WindowSize>) {}

    // async fn update(&mut self, _g:&mut G) {}
    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection);

    // // input handlers
    // async fn on_mouse_move(&mut self, _pos:Vector2, _g:&mut G) {}
    // async fn on_mouse_scroll(&mut self, _delta:f32, _g:&mut G) -> bool {false}
    // async fn on_mouse_down(&mut self, _pos:Vector2, _button:MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    // async fn on_mouse_up(&mut self, _pos:Vector2, _button:MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}

    // async fn on_text(&mut self, _text:&String) -> bool {false}
    // async fn on_key_press(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    // async fn on_key_release(&mut self, _key:Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}

    // async fn on_controller_press(&mut self, _controller: &GamepadInfo, _button: ControllerButton) -> bool {false}
    // async fn on_controller_release(&mut self, _controller: &GamepadInfo, _button: ControllerButton) -> bool {false}
    // async fn on_controller_axis(&mut self, _controller: &GamepadInfo, _axis_data: &HashMap<Axis, (bool, f32)>) {}

    // fn draw_background(&mut self, color:Color, offset: Vector2, list: &mut RenderableCollection) {
    //     let bounds = self.get_bounds();
    //     list.push(Rectangle::new(
    //         bounds.pos+offset,
    //         bounds.size,
    //         color.alpha(0.8),
    //         Some(Border::new(color, 2.0))
    //     ))
    // }
}
