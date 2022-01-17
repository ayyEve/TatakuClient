use crate::prelude::*;


/// a dialog is basically just a menu, except it does not occupy a whole gamemode,
/// and should be drawn overtop every other menu
pub trait Dialog<G> {
    fn update(&mut self, _g:&mut G) {}
    fn draw(&mut self, args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>);
    fn get_bounds(&self) -> Rectangle;
    fn should_close(&self) -> bool;
    fn name(&self) -> &'static str {""}

    // input handlers
    fn on_mouse_move(&mut self, _pos:&Vector2, _g:&mut G) {}
    fn on_mouse_scroll(&mut self, _delta:&f64, _g:&mut G) -> bool {false}
    fn on_mouse_down(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    fn on_mouse_up(&mut self, _pos:&Vector2, _button:&MouseButton, _mods:&KeyModifiers, _g:&mut G) -> bool {false}

    fn on_text(&mut self, _text:&String) -> bool {false}
    fn on_key_press(&mut self, _key:&Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
    fn on_key_release(&mut self, _key:&Key, _mods:&KeyModifiers, _g:&mut G) -> bool {false}
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