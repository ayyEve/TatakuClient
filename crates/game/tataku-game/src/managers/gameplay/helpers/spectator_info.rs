
#[derive(Default)]
pub struct GameplaySpectatorInfo {
    /// when was the last time the score was synchronized?
    pub last_score_sync: f32,

    /// who is currently spectating us?
    pub spectators: crate::prelude::engine::online::SpectatorList
}
