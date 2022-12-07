pub use crate::prelude::*;

#[async_trait]
pub trait AsyncMenu<G: Send+Sync>:Send+Sync {
    /// helpful for determining what menu this is
    fn get_name(&self) -> &str {"none"}
    async fn update(&mut self, _g:&mut G) {}
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>>;


    // input handlers
    async fn on_change(&mut self, _into:bool) {}// when the menu is "loaded"(into) or "unloaded"(!into)

    async fn on_text(&mut self, _text:String) {}
    async fn on_click(&mut self, _pos:Vector2, _button:MouseButton, _mods:KeyModifiers, _g:&mut G) {}
    async fn on_click_release(&mut self, _pos:Vector2, _button:MouseButton, _g:&mut G) {}

    async fn on_scroll(&mut self, _delta:f64, _g:&mut G) {}
    async fn on_mouse_move(&mut self, _pos:Vector2, _g:&mut G) {}
    async fn on_key_press(&mut self, _key:Key, _g:&mut G, _mods:KeyModifiers) {}
    async fn on_key_release(&mut self, _key:Key, _g:&mut G) {}
    async fn on_focus_change(&mut self, _has_focus:bool, _g:&mut G) {}


    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>);
}