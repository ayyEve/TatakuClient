use crate::prelude::*;

#[derive(Default, Clone, Debug)]
#[derive(Reflect)]
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

    pub fn update_values(&self, values: &mut dyn Reflect) {
        if let Err(e) = values.reflect_insert("global.lobbies", self.lobbies.values().cloned().collect::<Vec<_>>()) {
            error!("error updating global.lobbies: {e:?}");
        }

        // values.global.lobbies = self.lobbies.values().cloned().collect();
        // values.set("global.lobbies", TatakuVariable::new_game((TatakuVariableAccess::GameOnly, lobbies)));
    }
}

#[derive(Clone, Default, Debug)]
#[derive(Reflect)]
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


// impl Into<TatakuValue> for &CurrentLobbyInfo {
//     fn into(self) -> TatakuValue {
//         let mut lobby_map = HashMap::default();
//         lobby_map.set_value("id", TatakuVariable::new(self.id));
//         lobby_map.set_value("name", TatakuVariable::new(self.name.clone()));
//         lobby_map.set_value("host", TatakuVariable::new(self.host));
//         lobby_map.set_value("map", TatakuVariable::new(&self.current_beatmap));
//         lobby_map.set_value("state", TatakuVariable::new(format!("{:?}", self.state)));
//         lobby_map.set_value("is_host", TatakuVariable::new(self.is_host()));

//         let players = self.info.players.iter().map(|p| (p.user_id, p)).collect::<HashMap<_,_>>();
//         let mut slots = Vec::new();
//         for (id, slot) in self.info.slots.iter() {
//             let mut slot_map = HashMap::default();
//             slot_map.set_value("id", TatakuVariable::new((*id) as u32));

//             // states
//             slot_map.set_value("empty", TatakuVariable::new(slot == &LobbySlot::Empty));
//             slot_map.set_value("filled", TatakuVariable::new(false));
//             slot_map.set_value("locked", TatakuVariable::new(slot == &LobbySlot::Locked));

//             slot_map.set_value("status", TatakuVariable::new(format!("{slot:?}")));
//             slot_map.set_value("is_host", TatakuVariable::new(false));

//             if let LobbySlot::Filled { user } = slot {
//                 let username = self.player_usernames.get(user).cloned().unwrap_or(format!("[Loading...]"));
//                 slot_map.set_value("filled", TatakuVariable::new(true));
//                 slot_map.set_value("is_host", TatakuVariable::new(user == &self.host));

//                 // this should always resolve
//                 if let Some(player) = players.get(user) {
//                     let mut player_map = HashMap::default();
//                     let mods = ModManager::new().with_mods(player.mods.iter()).with_speed(player.speed);
//                     player_map.set_value("mods", TatakuVariable::new(mods));
//                     player_map.set_value("username", TatakuVariable::new(username));
//                     player_map.set_value("user_id", TatakuVariable::new(*user));

//                     player_map.set_value("no_map", TatakuVariable::new(player.state == LobbyUserState::NoMap));
//                     player_map.set_value("in_game", TatakuVariable::new(player.state == LobbyUserState::InGame));
//                     player_map.set_value("not_ready", TatakuVariable::new(player.state == LobbyUserState::NotReady));
//                     player_map.set_value("ready", TatakuVariable::new(player.state == LobbyUserState::Ready));

//                     let status = match player.state {
//                         LobbyUserState::NoMap => "No Map",
//                         LobbyUserState::InGame => "Playing",
//                         LobbyUserState::Ready => "Ready",
//                         LobbyUserState::NotReady => "Not Ready",
//                         LobbyUserState::Unknown => "???",
//                     };
//                     player_map.set_value("status", TatakuVariable::new(status));


//                     if user == &self.our_user_id {
//                         lobby_map.set_value("our_player", TatakuVariable::new(player_map.clone()));
//                     }

//                     slot_map.set_value("player", TatakuVariable::new(player_map));
//                 }

//                 if let Some(score) = self.player_scores.get(user) {
//                     slot_map.set_value("score", TatakuVariable::new(score));
//                 }
//             }

//             slots.push((*id, TatakuVariable::new(slot_map)));
//         }

//         slots.sort_by(|a, b| a.0.cmp(&b.0));
//         let slots = slots.into_iter().map(|(_,a)| a).collect::<Vec<_>>();

//         lobby_map.set_value("slots", TatakuVariable::new(TatakuValue::List(slots)));
//         TatakuValue::Map(lobby_map)
//     }
// }

impl From<&Option<LobbyBeatmap>> for TatakuValue {
    fn from(value: &Option<LobbyBeatmap>) -> Self {
        let mut map = HashMap::default();
        map.set_value("exists", TatakuVariable::new(false));
        let Some(beatmap) = value.as_ref() else { return TatakuValue::Map(map) };

        map.set_value("exists", TatakuVariable::new(true));
        map.set_value("hash", TatakuVariable::new(beatmap.hash.to_string()));
        map.set_value("playmode", TatakuVariable::new(&beatmap.mode));
        map.set_value("title", TatakuVariable::new(&beatmap.title));
        match &beatmap.map_game {
            MapGame::Osu => map.set_value("game", TatakuVariable::new("Osu")),
            MapGame::Quaver => map.set_value("game", TatakuVariable::new("Quaver")),
            MapGame::Other(other) => map.set_value("game", TatakuVariable::new(other)),
        }

        TatakuValue::Map(map)
    }
}

impl From<&LobbyInfo> for TatakuValue {
    fn from(value: &LobbyInfo) -> Self {
        let mut map = HashMap::default();
        map.set_value("id", TatakuVariable::new(value.id));
        map.set_value("name", TatakuVariable::new(&value.name));
        map.set_value("host", TatakuVariable::new(value.host));
        map.set_value("state", TatakuVariable::new(format!("{:?}", value.state)));
        map.set_value("players", TatakuVariable::new((TatakuVariableAccess::ReadOnly, value.players.clone())));

        // map.set("map", &value.current_beatmap);
        // map.set("players", &value.players);

        TatakuValue::Map(map)
    }
}
impl From<LobbyInfo> for TatakuValue {
    fn from(value: LobbyInfo) -> Self {
        (&value).into()
    }
}