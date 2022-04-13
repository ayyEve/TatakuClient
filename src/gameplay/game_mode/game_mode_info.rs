use crate::prelude::*;

/// not sure "Info" is the right word but whatever
#[async_trait]
pub trait GameModeInfo: Send + Sync {
    /// playmode for this game mode
    fn playmode(&self) -> PlayMode;
    /// should the cursor be visible (ie, osu yes, taiko/mania no)
    fn show_cursor(&self) -> bool {false}
    
    /// what ms does this map end?
    fn end_time(&self) -> f32;

    /// turn a score hit into a string. used for the judgement counter and score screen
    fn score_hit_string(hit:&ScoreHit) -> String where Self: Sized;

    /// what key presses are valid, as well as what they should be named as
    /// used for the key counter
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)>;

    /// setup any gamemode specific ui elements for this gamemode
    /// ie combo and leaderboard, since the pos is different per-mode
    async fn get_ui_elements(&self, _window_size: Vector2, _ui_elements: &mut Vec<UIElement>) {}
    
    /// f64 is hitwindow, color is color for that window. last is miss hitwindow
    /// TODO: change this to a Vec<(f32, ScoreHit, Color)>, where the f32 is the hit window and color is the color for that hit window
    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color));
}