use crate::prelude::*;

// pub type MultiplayerDataHelper = GlobalValue<MultiplayerData>;
// pub type CurrentLobbyDataHelper = GlobalValue<Option<CurrentLobbyInfo>>;

#[derive(Default, Clone, Debug)]
pub struct MultiplayerData {
    pub lobbies: HashMap<u32, LobbyInfo>,

    pub lobby_creation_pending: bool,
    pub lobby_join_pending: bool,
}
impl MultiplayerData {
    pub fn clear(&mut self) {
        self.lobbies.clear();
        self.lobby_creation_pending = false;
        self.lobby_join_pending = false;
    }

    pub fn update_values(&self, values: &mut ValueCollection) {
        let lobbies = self.lobbies.values().collect::<Vec<_>>();
        values.set("global.lobbies", lobbies);
    }
}

#[derive(Clone, Default, Debug)]
pub struct CurrentLobbyInfo {
    /// what is our user id?
    pub our_user_id: u32,

    /// lobby information
    pub info: FullLobbyInfo,

    /// should we be loading the map?
    pub play_pending: bool,

    /// should we start playing the map?
    pub should_play: bool,

    /// scores of the players in the lobby
    pub player_scores: HashMap<u32, Score>,

    /// cache of lobby player usernames
    pub player_usernames: HashMap<u32, String>,
}
impl CurrentLobbyInfo {
    pub fn new(info: FullLobbyInfo, our_user_id: u32) -> Self {
        Self {
            our_user_id,
            info, 
            play_pending: false,
            should_play: false,
            player_scores: HashMap::new(),
            player_usernames: HashMap::new(),
        }
    }
    
    pub fn is_host(&self) -> bool {
        self.host == self.our_user_id
    }
    pub fn our_user(&self) -> Option<&LobbyUser> {
        self.players.iter().find(|u| u.user_id == self.our_user_id)
    }

    pub async fn update_usernames(&mut self) {
        self.player_usernames.clear();
        let om = OnlineManager::get().await;
        
        for user in self.info.players.iter() {
            let Some(user) = om.users.get(&user.user_id) else { continue };
            let user = user.lock().await;
            self.player_usernames.insert(user.user_id, user.username.clone());
        }
    }

}

impl Deref for CurrentLobbyInfo {
    type Target = FullLobbyInfo;
    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
impl DerefMut for CurrentLobbyInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}


impl Into<CustomElementValue> for &CurrentLobbyInfo {
    fn into(self) -> CustomElementValue {
        let mut lobby_map = CustomElementMapHelper::default();
        lobby_map.set("id", self.id);
        lobby_map.set("name", self.name.clone());
        lobby_map.set("host", self.host);
        lobby_map.set("map", &self.current_beatmap);
        lobby_map.set("state", format!("{:?}", self.state));
        lobby_map.set("is_host", self.is_host());

        let players = self.info.players.iter().map(|p| (p.user_id, p)).collect::<HashMap<_,_>>();
        let mut slots = Vec::new();
        for (id, slot) in self.info.slots.iter() {
            let mut slot_map = CustomElementMapHelper::default();
            slot_map.set("id", (*id) as u32);

            // states
            slot_map.set("empty", slot == &LobbySlot::Empty);
            slot_map.set("filled", false);
            slot_map.set("locked", slot == &LobbySlot::Locked);

            slot_map.set("status", format!("{slot:?}"));
            slot_map.set("is_host", false);

            if let LobbySlot::Filled { user } = slot { 
                let username = self.player_usernames.get(user).cloned().unwrap_or(format!("[Loading...]"));
                slot_map.set("filled", true);
                slot_map.set("is_host", user == &self.host);

                // this should always resolve
                if let Some(player) = players.get(user) {
                    let mut player_map = CustomElementMapHelper::default();
                    let mods = ModManager::new().with_mods(player.mods.clone()).with_speed(player.speed);
                    player_map.set("mods", mods);
                    player_map.set("username", username);
                    player_map.set("user_id", *user);

                    player_map.set("no_map", player.state == LobbyUserState::NoMap);
                    player_map.set("in_game", player.state == LobbyUserState::InGame);
                    player_map.set("not_ready", player.state == LobbyUserState::NotReady);
                    player_map.set("ready", player.state == LobbyUserState::Ready);

                    let status = match player.state {
                        LobbyUserState::NoMap => "No Map",
                        LobbyUserState::InGame => "Playing",
                        LobbyUserState::Ready => "Ready",
                        LobbyUserState::NotReady => "Not Ready",
                        LobbyUserState::Unknown => "???",
                    };
                    player_map.set("status", status);


                    let player_map = player_map.finish();
                    if user == &self.our_user_id {
                        lobby_map.set("our_player", player_map.clone());
                    }

                    slot_map.set("player", player_map);
                }
            
                if let Some(score) = self.player_scores.get(user) {
                    slot_map.set("score", score);
                }
            }

            slots.push((*id, slot_map.finish()));
        }

        slots.sort_by(|a, b| a.0.cmp(&b.0));
        let slots = slots.into_iter().map(|(_,a)| a).collect::<Vec<_>>();

        lobby_map.set("slots", slots);
        lobby_map.finish()
    }
}

impl From<&Option<LobbyBeatmap>> for CustomElementValue {
    fn from(value: &Option<LobbyBeatmap>) -> Self {
        let mut map = CustomElementMapHelper::default();
        map.set("exists", false);
        let Some(beatmap) = value.as_ref() else { return map.finish() };

        map.set("exists", true);
        map.set("hash", beatmap.hash);
        map.set("playmode", &beatmap.mode);
        map.set("title", &beatmap.title);
        match &beatmap.map_game {
            MapGame::Osu => map.set("game", "Osu"),
            MapGame::Quaver => map.set("game", "Quaver"),
            MapGame::Other(other) => map.set("game", other),
        }

        map.finish()
    }
}

impl From<&LobbyInfo> for CustomElementValue {
    fn from(value: &LobbyInfo) -> Self {
        let mut map = CustomElementMapHelper::default();
        map.set("id", value.id);
        map.set("name", &value.name);
        map.set("host", value.host);
        map.set("state", format!("{:?}", value.state));
        map.set("players", value.players.clone());

        // map.set("map", &value.current_beatmap);
        // map.set("players", &value.players);

        map.finish()
    }
}
impl From<LobbyInfo> for CustomElementValue {
    fn from(value: LobbyInfo) -> Self {
        (&value).into()
    }
}