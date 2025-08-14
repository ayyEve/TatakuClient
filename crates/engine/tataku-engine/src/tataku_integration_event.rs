use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum TatakuIntegrationEvent {

    /// user started playing a beatmap
    BeatmapStarted {
        /// what unix timestamp the beatmap was started at
        start_time: u64,

        /// beatmap that was started
        beatmap: Arc<BeatmapMeta>,

        /// what mode the user is playing
        playmode: ArcStr,

        /// multiplayer lobby info
        multiplayer: Option<LobbyInfo>,

        /// username of who's being spectated
        spectator: Option<ArcStr>
    },

    SongChanged {
        artist: ArcStr,
        title: ArcStr,
        image_path: ArcStr,
        elapsed: f32,
        duration: f32,
    },

    /// beatmap has ended
    BeatmapEnded,

    /// user joined a multiplayer lobby
    JoinedMultiplayer(LobbyInfo),

    /// user left the multiplayer lobby
    LeftMultiplayer,

    /// name of the menu entered
    MenuEntered(CowStr),
}
