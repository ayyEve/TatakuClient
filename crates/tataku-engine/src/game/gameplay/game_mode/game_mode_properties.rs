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
    async fn get_ui_elements(&self, _window_size: Vector2, _ui_elements: &mut Vec<UIElement>) {}
    
    /// f32 is hitwindow, color is color for that window
    fn timing_bar_things(&self) -> Vec<(f32, Color)>;
    
    fn get_info(&self) -> Arc<dyn GameModeInfo>;
}