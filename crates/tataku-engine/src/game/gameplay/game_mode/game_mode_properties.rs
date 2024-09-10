use crate::prelude::*;

#[async_trait]
pub trait GameModeProperties: Send + Sync {
    /// playmode for this game mode
    fn playmode(&self) -> Cow<'static, str>;
    /// should the cursor be visible (ie, osu yes, taiko/mania no)
    fn show_cursor(&self) -> bool { false }
    
    /// what ms does this map end?
    fn end_time(&self) -> f32;

    /// what key presses are valid, as well as what they should be named as
    /// used for the key counter
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)>;

    /// setup any gamemode specific ui elements for this gamemode
    /// ie combo and leaderboard, since the pos is different per-mode
    async fn get_ui_elements(
        &self, 
        _window_size: Vector2, 
        _ui_elements: &mut Vec<UIElement>,
        _loader: &mut dyn UiElementLoader,
    ) {}
    
    /// f32 is hitwindow, color is color for that window
    fn timing_bar_things(&self) -> Vec<(f32, Color)>;
    
    fn get_info(&self) -> GameModeInfo;
}


#[async_trait]
pub trait UiElementLoader: Send + Sync {
    async fn load(
        &mut self, 
        name: &str, 
        default_pos: Vector2, 
        inner: Box<dyn InnerUIElement>
    ) -> UIElement;
}

pub struct DefaultUiElementLoader;

#[async_trait]
impl UiElementLoader for DefaultUiElementLoader {
    async fn load(
        &mut self, 
        name: &str, 
        default_pos: Vector2, 
        inner: Box<dyn InnerUIElement>
    ) -> UIElement {
        let element_name = name.to_owned();
        let mut pos_offset = default_pos;
        let mut scale = Vector2::ONE;
        let mut visible = true;
        
        if let Some((stored_pos, stored_scale, stored_window_size, stored_visible)) = Database::get_element_info(&element_name).await {
            pos_offset = stored_pos;
            scale = stored_scale;
            visible = stored_visible;
            
            if stored_window_size.length() > 0.0 {
                // debug!("got stored window size {stored_window_size:?}");
                do_scale(&mut pos_offset, &mut scale, stored_window_size, WindowSize::get().0);
            }
        }

        if scale.x.abs() < 0.01 { scale.x = 1.0 }
        if scale.y.abs() < 0.01 { scale.y = 1.0 }

        UIElement {
            default_pos,
            element_name,
            pos_offset,
            scale,
            inner,
            visible
        }

    }
}

#[allow(unused)]
fn do_scale(pos: &mut Vector2, scale: &mut Vector2, old_window_size: Vector2, new_window_size: Vector2) {
    // TODO:
    // let new_scale = new_window_size / old_window_size;
    // let scaled_pos_offset = new_window_size - old_window_size * new_scale;

    // *pos = scaled_pos_offset + *pos * new_scale;
    // *scale *= new_scale
}