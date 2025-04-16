use crate::prelude::*;

#[derive(Reflect)]
#[derive(Debug, Default)]
#[reflect(dont_clone)]
// #[reflect(remap("map" => "self.beatmap_manager.current_beatmap.map"))]
pub struct GameValues {
    pub settings: Settings,

    pub song: SongInfo,
    pub game: GameInfo,
    pub global: GlobalInfo,
    pub enums: EnumInfo,
    pub theme: Theme,

    pub lobby: Option<CurrentLobbyInfo>,
    pub score: IngameScore,

    /// Beatmap manager, its here instead of in Game to keep the lists in one place
    #[reflect(alias("beatmaps"))] pub beatmap_manager: BeatmapManager,

    /// Online manager, its here instead of in Game to keep the lists in one place
    #[reflect(alias("online"))] pub online_manager: OnlineManager,

    /// list of retreived scored 
    #[reflect(alias("scores_list"))] pub score_list: ScoreList,
    #[reflect(alias("downloads"))] pub download_manager: DownloadManager,
}
impl GameValues {
    pub fn new(
        infos: GamemodeInfos, 
        settings: &Settings
    ) -> Self {
        Self {
            enums: EnumInfo::new(&infos),
            settings: settings.clone(),
            beatmap_manager: BeatmapManager::new(infos.clone()),
            global: GlobalInfo::new(infos.clone(), settings),
            ..Default::default()
        }
    }

    pub fn current_beatmap_prop<T>(&self, f: impl FnOnce(&BeatmapMeta)->T) -> Option<T> {
        self
            .beatmap_manager
            .current_beatmap
            .as_ref()
            .map(|b| f(b))
    }
}


#[derive(Debug, Clone, Default)]
#[derive(Reflect)]
pub struct ScoreList {
    #[reflect(flatten)]
    pub scores: Vec<IngameScore>,
    pub loaded: bool,
}


#[derive(Default)]
pub struct ValueCollection {
    pub values: GameValues,
    pub custom: DynMap,
}
impl Deref for ValueCollection {
    type Target = GameValues;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl DerefMut for ValueCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

// TODO: forward the error from values and not the dynmap (or both?)
impl Reflect for ValueCollection {
    fn impl_get<'v, 's>(&'s self, path: ReflectPath<'v>) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        self
            .values
            .impl_get(path.clone())
            .or_else(|_| self.custom.impl_get(path))
    }

    fn impl_get_mut<'v>(&mut self, path: ReflectPath<'v>) -> ReflectResult<'v, &mut dyn Reflect> {
        self.values.impl_get_mut(path.clone())
            .or_else(|_| self.custom.impl_get_mut(path))
    }

    fn impl_insert<'v>(&mut self, path: ReflectPath<'v>, value: Box<dyn Reflect>) -> ReflectResult<'v, ()> {
        if self.values.impl_get(path.clone()).is_ok() {
            self.values.impl_insert(path, value)
        } else {
            self.custom.impl_insert(path, value)
        }
    }

    fn impl_iter<'v>(&self, path: ReflectPath<'v>) -> ReflectResult<'v, ReflectIter<'_>> {
        match (self.values.impl_iter(path.clone()), self.custom.impl_iter(path)) {
            (Ok(v), Ok(c)) => Ok(v.chain(c).collect::<Vec<_>>().into()),
            (Ok(v), Err(_)) => Ok(v),
            (Err(_), Ok(c)) => Ok(c),
            (Err(ReflectError::EntryNotExist { .. }), Err(e)) => Err(e),
            (Err(e), Err(ReflectError::EntryNotExist { .. })) => Err(e),
            // TODO: is this correct?
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn impl_iter_mut<'v>(&mut self, path: ReflectPath<'v>) -> ReflectResult<'v, ReflectIterMut<'_>> {
        match (self.values.impl_iter_mut(path.clone()), self.custom.impl_iter_mut(path)) {
            (Ok(v), Ok(c)) => Ok(v.chain(c).collect::<Vec<_>>().into()),
            (Ok(v), Err(_)) => Ok(v),
            (Err(_), Ok(c)) => Ok(c),
            (Err(ReflectError::EntryNotExist { .. }), Err(e)) => Err(e),
            (Err(e), Err(ReflectError::EntryNotExist { .. })) => Err(e),
            // TODO: is this correct?
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> { None }

    fn from_string(_: &str) -> ReflectResult<'_, Box<dyn Reflect>> where Self:Sized {
        Err(ReflectError::NoFromString)
    }
}


#[derive(Reflect)]
#[derive(Default, Debug, Copy, Clone)]
#[reflect(display = "debug")]
pub struct SongInfo {
    pub position: f32,
    pub paused: bool,
    pub playing: bool,
    pub stopped: bool,
    pub exists: bool,

    pub state: AudioState,
}
impl SongInfo {
    pub fn update(&mut self, audio: Option<Arc<dyn AudioInstance>>) {
        if let Some(audio) = audio {
            self.position = audio.get_position();
            self.set_state(audio.get_state());
            self.exists = true;
        } else {
            self.position = 0.0;
            self.set_state(AudioState::Unknown);
            self.exists = false;
        }
    }
    pub fn set_state(&mut self, state: AudioState) -> bool {
        if self.state == state { return false }

        self.paused = state == AudioState::Paused;
        self.playing = state == AudioState::Playing;
        self.stopped = state == AudioState::Stopped;
        self.exists = self.paused || self.playing || self.stopped;

        self.state = state;

        true
    }
}

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Default, Debug, Copy, Clone)]
pub struct GameInfo {
    pub time: f32,
    pub window_size: Vector2,
}


#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Default, Debug, Clone)]
pub struct GlobalInfo {
    pub mods: ModManager,

    pub lobbies: Vec<LobbyInfo>,

    #[reflect(alias("infos"))]
    pub gamemode_infos: GamemodeInfos,

    pub playmode: String,
    pub playmode_display: String,
    pub playmode_actual: String,
    pub playmode_actual_display: String,

    pub username: String,
    pub user_id: u32,
    pub logged_in: bool,
    pub menu_list: Vec<String>,

    pub new_beatmap_hash: Option<Md5Hash>,
}
impl GlobalInfo {
    pub fn new(
        infos: GamemodeInfos,
        settings: &Settings,
    ) -> Self {
        let mut s = Self {
            gamemode_infos: infos,
            username: settings.username.clone(),
            ..Default::default()
        };
        s.update_playmode(settings.last_played_mode.clone());
        s.update_playmode_actual(settings.last_played_mode.clone());
        
        s
    }

    pub fn update_playmode(
        &mut self, 
        playmode: String,
    ) {
        self.playmode = playmode.clone();
        let Ok(info) = self.gamemode_infos.get_info(&playmode) else { return };
        self.playmode_display = info.display_name.to_owned();
    }
    pub fn update_playmode_actual(
        &mut self, 
        playmode: String,
    ) {
        self.playmode_actual = playmode.clone();
        let Ok(info) = self.gamemode_infos.get_info(&playmode) else { return };
        self.playmode_actual_display = info.display_name.to_owned();
    }
}


#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Debug, Clone, Default)]
pub struct EnumInfo {
    pub sort_by: Vec<SortBy>,
    pub group_by: Vec<GroupBy>,
    pub score_methods: Vec<ScoreRetreivalMethod>,

    pub playmodes: Vec<String>,
    pub playmodes_display: Vec<String>,
}
impl EnumInfo {
    pub fn new(infos: &GamemodeInfos) -> Self {
        let playmodes = infos.by_num.iter().map(|g| g.id.to_string()).collect::<Vec<_>>();
        let playmodes_display = infos.by_num.iter()
            .map(|s| s.display_name.to_owned())
            .collect();

        Self {
            sort_by: SortBy::list(),
            group_by: GroupBy::list(),
            score_methods: ScoreRetreivalMethod::list(),

            playmodes,
            playmodes_display,
        }
    }
}
