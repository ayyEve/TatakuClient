use crate::prelude::*;
use tataku::Vector2;
use common::{
    reflect::*,
    network::multiplayer::*,
};

use engine::{
    BeatmapMeta,
    online_content::{
        OnlineContentItem,
        OnlineContentCapabilities,
    },
    gameplay::{
        IngameScore,
        GamemodeInfo,
        GamemodeInfos,
    },
};

#[derive(Reflect)]
#[reflect(dont_clone)]
#[derive(Debug, Default)]
// #[reflect(remap(map => self.beatmap_manager.current_beatmap.map))]
pub struct TatakuValues {

    /// The Game's settings
    pub settings: engine::Settings,

    /// The current song information
    pub song: SongInfo,

    /// Game values
    pub game: GameValues,

    /// Global values
    pub global: GlobalValues,

    /// Enums and their variants
    pub enums: EnumValues,

    /// The current Tataku theme
    #[cfg(feature="graphics")] 
    pub theme: graphics::Theme,

    /// The current score
    pub score: ReflectScore,

    /// The multiplayer lobby, if we're in one
    #[cfg(feature="gameplay")] 
    pub lobby: Option<ReflectLobby>,

    /// Beatmap manager, its here instead of in Game to keep the lists in one place
    #[reflect(alias("beatmaps"))] 
    pub beatmap_manager: BeatmapManager,
    
    /// beatmap settings
    pub beatmap_settings: BeatmapSettings,

    /// Online manager, its here instead of in Game to keep the lists in one place
    #[reflect(alias("online"))] 
    #[cfg(feature="gameplay")]
    pub online_manager: OnlineManager,

    /// List of retreived scores
    #[reflect(alias("scores_list"))] 
    pub score_list: ScoreList,

    /// The download manager
    #[reflect(alias("downloads"))] 
    pub download_manager: DownloadManager,
}
impl TatakuValues {
    pub fn new(
        infos: &GamemodeInfos, 
        online_content_engines: Vec<OnlineContentCapabilities>,
        settings: engine::Settings,
    ) -> Self {
        Self {
            enums: EnumValues::new(infos),
            #[cfg(feature="gameplay")]
            online_manager: OnlineManager::new(),
            game: GameValues::new(online_content_engines),
            global: GlobalValues::new(infos.clone(), &settings),
            beatmap_manager: BeatmapManager::new(infos.clone()),
            settings,
            ..Default::default()
        }
    }

    pub fn current_beatmap_prop<T>(
        &self, 
        f: impl FnOnce(&BeatmapMeta) -> T
    ) -> Option<T> {
        self
            .beatmap_manager
            .current_beatmap()
            .map(|b| f(b))
    }
}



#[derive(Reflect)]
#[derive(Debug, Clone)]
#[cfg(feature="gameplay")]
pub struct ReflectLobby {
    /// scores of the players in the lobby
    player_scores: Vec<ReflectScore>,

    /// lobby id
    id: u32,

    /// name of the lobby
    name: String,
    
    /// who is the current host
    host: u32,
    
    /// current state of the lobby
    state: LobbyState,

    /// ids of the users in this lobby
    players: Vec<LobbyUser>,

    /// slot states
    slots: Vec<LobbySlot>,

    /// title of the current beatmap
    current_beatmap: Option<LobbyBeatmap>,
}
#[cfg(feature="gameplay")]
impl ReflectLobby {
    pub fn new(lobby: &CurrentLobbyInfo) -> Self {
        Self {
            player_scores: Vec::new(),
            id: lobby.id,
            name: lobby.name.clone(),
            host: lobby.host,
            state: lobby.state,
            players: lobby.players.clone(),
            slots: lobby.slots.values().copied().collect(),
            current_beatmap: lobby.current_beatmap.clone(),
        }
    }
    
    pub fn update(
        &mut self, 
        lobby: &CurrentLobbyInfo, 
        info: &GamemodeInfo
    ) {
        // FIXME: this is bad
        self.player_scores = lobby.player_scores
            .values()
            .map(|s| ReflectScore::new(&IngameScore::new(
                s.clone(), 
                false, 
                false
            ), info))
            .collect();

        self.host = lobby.host;
        self.state = lobby.state;
        self.players = lobby.players.clone();
        self.slots = lobby.slots.values().copied().collect();
        self.current_beatmap = lobby.current_beatmap.clone();
    }
}


#[derive(Reflect)]
#[derive(Debug, Clone, Default)]
pub struct ScoreList {
    #[reflect(flatten)]
    pub scores: Vec<IngameScore>,
    pub loaded: bool,
}



#[derive(Reflect)]
#[derive(Default, Debug)]
#[reflect(display = "debug", dont_clone)]
pub struct GameValues {
    pub time: f32,
    pub window_size: Vector2,
    pub loading_statuses: Vec<LoadingStatus>,
    pub online_content: OnlineContentValues,
}
impl GameValues {
    pub fn new(online_content_engines: Vec<OnlineContentCapabilities>) -> Self {
        Self {
            online_content: OnlineContentValues::new(online_content_engines),
            ..Default::default()
        }
    }
}

#[derive(Reflect)]
#[derive(Default, Debug)]
#[reflect(display = "debug", dont_clone)]
pub struct OnlineContentValues {
    pub engines: HashMap<String, OnlineContentCapabilities>,
    pub results: OnlineContentReflectResults,
}
impl OnlineContentValues {
    pub fn new(engines: Vec<OnlineContentCapabilities>) -> Self {
        let engines = engines
            .into_iter()
            .enumerate()
            .flat_map(|(n, i)| [
                (i.engine_id.clone(), i.clone()),
                // workaround/compatability,
                // allows engines to be indexed by number
                (n.to_string(), i) 
            ])
            .collect::<HashMap<_,_>>();

        Self {
            engines,
            results: OnlineContentReflectResults::default(),
        }
    }
}




#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Default, Debug, Clone)]
pub struct OnlineContentReflectResults {
    pub completed: bool,
    pub error: Option<String>,
    pub items: Vec<OnlineContentItem>,
    pub count: usize,
    pub page: usize,
}
