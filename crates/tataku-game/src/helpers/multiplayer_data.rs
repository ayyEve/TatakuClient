use crate::prelude::*;

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Default, Clone, Debug)]
pub struct MultiplayerData {
    pub lobby_creation_pending: bool,
    pub lobby_join_pending: bool,
}
impl MultiplayerData {
    pub fn clear(&mut self) {
        self.lobby_creation_pending = false;
        self.lobby_join_pending = false;
    }
}

#[derive(Reflect)]
#[reflect(display = "debug")]
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

