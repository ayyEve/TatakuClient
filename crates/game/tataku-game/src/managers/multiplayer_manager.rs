use crate::prelude::*;
use tataku::Color;
use common::{
    Score,
    Md5Hash,
    GameSpeed,
    reflect::*,

    network::multiplayer::*,
    packets::{
        PacketId,
        MultiplayerPacket,
    },
};

use engine::{
    actions,
    Notification,
    data::ValueChange,
    actions::{
        mods::ModAction as ModAction,
        menu::MenuAction as MenuAction,
        online::OnlineAction as OnlineAction,
        beatmap::{
            SetBeatmapOptions,
            BeatmapAction as BeatmapAction,
        },
        multiplayer::{
            LobbyAction as LobbyAction,
            LobbySlotAction as LobbySlotAction,
            MultiplayerAction as MultiplayerAction,
        }
    },
    gameplay::{
        IngameScore,
        GamemodeInfos,
        mods::Mods,
        gameplay_manager::GameplayManagerTrait,
    },
};

#[derive(Debug2)]
pub struct MultiplayerManager {
    /// lobby data
    pub lobby: CurrentLobbyInfo,

    /// what is the current beatmap we have selected?
    current_beatmap: ValueChange<Md5Hash>,

    /// what playmode is selected by the host?
    selected_mode: Option<ArcStr>,

    /// what mods are currently enabled?
    current_mods: ValueChange<Mods>,

    new_beatmap: ValueChange<Md5Hash>,

    // /// async beatmap loader
    // beatmap_loader: Option<engine::io::AsyncLoader<tataku::Result<GameplayManager>>>,
    #[debug(skip)]
    beatmap_loader: Option<tataku::Result<GameplayManager>>,

    /// have we sent that we've loaded the beatmap?
    load_complete_sent: bool,

    /// have we sent a skip request?
    skip_request_sent: bool,

    infos: GamemodeInfos,
}

impl MultiplayerManager {
    pub fn new(
        lobby: CurrentLobbyInfo,
        infos: GamemodeInfos,
        actions: &mut actions::ActionQueue,
    ) -> Self {
        use actions::beatmap::{
            BeatmapAction as BeatmapAction,
            SetBeatmapOptions,
        };

        // make sure our game is up to date with the lobby's current info
        match lobby.current_beatmap.clone() {
            Some(map) => {
                actions.push(BeatmapAction::Set(
                    map.hash,
                    SetBeatmapOptions::default().restart_song(false)
                ).into());
                actions.push(BeatmapAction::SetPlaymode(map.mode).into());
            }
            None => {
                actions.push(BeatmapAction::Remove.into());
            }
        }

        Self {
            lobby,
            infos,
            current_beatmap: ValueChange::new("beatmaps.current.map.beatmap_hash"),
            selected_mode: None,
            current_mods: ValueChange::new("global.mods"),

            new_beatmap: ValueChange::new("global.new_map_hash"),

            beatmap_loader: None,
            load_complete_sent: false,
            skip_request_sent: false,
        }
    }

    #[allow(clippy::needless_pass_by_value, reason = "its ref internally??")]
    pub fn update(
        &mut self,
        manager: Option<&mut Box<GameplayManager>>,
        values: &mut ValueCollection,
        actions: &mut actions::ActionQueue,
    ) {
        let previous_map = self.current_beatmap.clone();

        match self.current_beatmap.update(values)
            .map(|i| i.copied())
            .map_err(|e| e.to_owned()) {
            Err(ReflectError::OptionIsNone) if self.is_host() => {
                // if nothing was selected, make sure we revert back to the previous beatmap
                if let Some(old_map) = *previous_map {
                    warn!("selecting previous map");
                    actions.push(BeatmapAction::Set(
                        old_map,
                        SetBeatmapOptions::default()
                            .restart_song(false)
                            .use_preview_point(true)
                    ).into());
                    self.set_state(LobbyUserState::NotReady, actions);
                }
            }


            // if we're the host, the map update was us selecting a map
            // so, update the lobby with the selected map
            Ok(Some(map)) if self.is_host() => {
                if !self.current_beatmap_is_selected() {
                    actions.push(MultiplayerAction::SetBeatmap {
                        hash: map,
                        mode: self.selected_mode.clone()
                    }.into());
                    self.set_state(LobbyUserState::NotReady, actions);
                }
            }

            Err(ReflectError::OptionIsNone) => {
                // not host, dont have map.
                self.set_state(LobbyUserState::NoMap, actions);
            }
            Ok(Some(_)) => {
                self.set_state(LobbyUserState::NotReady, actions);
            }


            Err(_e) => {
                // error!("{e:?}");
            }

            _ => {}
        }

        // if we're loading the beatmap, check if its done
        if let Some(loader) = &self.beatmap_loader
        && !self.load_complete_sent && loader.is_ok() {
            self.load_complete_sent = true;
            self.send_packet(
                MultiplayerPacket::Client_LobbyMapLoaded,
                actions
            );
        }

        // if our mods changed, let the lobby know
        if let Ok(Some(mods)) = self.current_mods.update(values) {
            let speed = mods.speed;
            let mods = mods.mods.clone();
            self.send_packet(
                MultiplayerPacket::Client_LobbyUserModsChanged {
                    mods,
                    speed: speed.as_u8(),
                },
                actions
            );
        }

        // check if a new beatmap was added
        if let Some(Some(new_hash)) = self.new_beatmap.update(values).ok().filter(|_| manager.is_none()) {

            // if the map that was just added is the lobby's map, set it as our current map
            if let Some(beatmap) = &self.lobby.current_beatmap
            && new_hash == &beatmap.hash {
                actions.push(BeatmapAction::Set(
                    beatmap.hash,
                    SetBeatmapOptions::default().restart_song(true)
                ).into());
                self.set_state(LobbyUserState::NotReady, actions);
            }
        }
    }

    pub fn update_values(&self, values: &mut ValueCollection) {


        if let Some(lobby) = values.values.lobby.as_mut() {
            if let Some(playmode) = self.lobby.current_beatmap.as_ref().and_then(|b| values.values.global.gamemode_infos.get_info(&b.mode).ok()) {
                lobby.update(&self.lobby, playmode);
            }
        } else {
            values.lobby = Some(ReflectLobby::new(&self.lobby));
        }



        let Some(our_user) = self.lobby.our_user() else { return };

        let values = values.as_dyn_mut();
        values.reflect_insert("lobby.is_host", self.is_host()).unwrap();
        values.reflect_insert("lobby.our_user", our_user.clone()).unwrap();
        values.reflect_insert("lobby.our_user_state", format!("{:?}", our_user.state).to_lowercase()).unwrap();

        values.reflect_insert("lobby.map", self.lobby.current_beatmap.clone()).unwrap();
        values.reflect_insert("lobby.we_have_beatmap", self.current_beatmap.is_some()).unwrap();


        // TODO: this is shit
        {
            #[derive(Reflect, Clone, Default)]
            struct LobbySlotReflect {
                id: u8,
                empty: bool,
                filled: bool,
                locked: bool,
                is_host: bool,
                #[reflect(alias("user"))]
                player: Option<LobbySlotPlayerReflect>,
            }
            #[derive(Reflect, Clone, Default)]
            struct LobbySlotPlayerReflect {
                username: String,
                user_id: u32,
                state: String,
                status: String,
                has_map: bool,
            }

            let mut list = Vec::new();
            for slot_num in 0..self.lobby.slots.len() as u8 {
                let slot = self.lobby.slots.get(&slot_num).unwrap();
                let mut data = LobbySlotReflect {
                    id: slot_num,
                    ..Default::default()
                };

                match slot {
                    LobbySlot::Empty => data.empty = true,
                    LobbySlot::Filled { user } => {
                        let state = self.lobby.players.iter()
                            .find(|u| u.user_id == *user)
                            .map(|s| s.state)
                            .unwrap_or_default();

                        data.filled = true;
                        data.player = Some(LobbySlotPlayerReflect {
                            username: self.lobby.player_usernames.get(user).cloned().unwrap_or_default(),
                            user_id: *user,
                            state: format!("{state:?}"),
                            status: match state {
                                LobbyUserState::NoMap => "No Map",
                                LobbyUserState::InGame => "Playing",
                                LobbyUserState::Ready => "Ready",
                                LobbyUserState::NotReady => "Not Ready",
                                LobbyUserState::Unknown => "",
                            }.to_owned(),

                            has_map: state.has_map(),
                        });
                    },
                    LobbySlot::Locked => data.locked = true,
                    LobbySlot::Unknown => {},
                }

                list.push(data);
            }

            values.reflect_insert("lobby.player_slots", list).unwrap();
        }

    }


    fn current_beatmap_is_selected(&self) -> bool {
        let Some(current_map) = self.current_beatmap.as_ref() else { return false };
        let Some(selected) = &self.lobby.current_beatmap else { return false };
        &selected.hash == current_map
    }

    pub fn handle_packet(
        &mut self,
        values: &mut ValueCollection,
        packet: &MultiplayerPacket,
        manager: Option<&mut Box<GameplayManager>>,
        actions: &mut actions::ActionQueue,
        database: &dyn engine::database::DatabaseProvider,
    ) -> tataku::Result<Option<GameplayManager>> {
        match packet {
            MultiplayerPacket::Server_LobbyUserJoined { lobby_id, user_id } => {
                if &self.lobby.info.id != lobby_id { return Ok(None) }
                self.lobby.info.players.push(LobbyUser { user_id: *user_id, ..Default::default() });

                let Some(user) = values.online_manager.get_user(*user_id) else {
                    actions.push(
                        Notification::default()
                        .text(format!("User with id {user_id} joined the match"))
                        .duration(3000.0)
                        .color(Color::PURPLE)
                        .into()
                    );
                    self.update_values(values);
                    return Ok(None)
                };

                actions.push(
                    Notification::default()
                    .text(format!("{} joined the match", user.username))
                    .duration(3000.0)
                    .color(Color::PURPLE)
                    .into()
                );

                self.lobby.player_usernames.insert(*user_id, user.username);
            }
            MultiplayerPacket::Server_LobbyUserLeft { lobby_id, user_id } => {
                if &self.lobby.id != lobby_id { return Ok(None); }

                self.lobby.players.retain(|u| &u.user_id != user_id);

                // find the slot that had this user and set it to empty (server will update its proper status next update)
                if let Some(slot) = self.lobby.slots.values_mut().find(|s| **s == LobbySlot::Filled { user: *user_id }) {
                    *slot = LobbySlot::Empty;
                }

                if user_id != &self.lobby.our_user_id {
                    let username = self.lobby.player_usernames.remove(user_id).unwrap_or_default();
                    actions.push(
                        Notification::default()
                        .text(format!("{username} left the match"))
                        .duration(3000.0)
                        .color(Color::PURPLE)
                        .into()
                    );
                }
            }

            MultiplayerPacket::Server_LobbySlotChange { slot, new_status } => {
                if let Some(slot) = self.lobby.info.slots.get_mut(slot) {
                    *slot = *new_status;
                }
            }

            MultiplayerPacket::Server_LobbyUserState { user_id, new_state } => {
                if let Some(user) =
                self.lobby.info.players
                    .iter_mut()
                    .find(|u| &u.user_id == user_id) {
                    user.state = *new_state;
                }
            }


            MultiplayerPacket::Server_LobbyStart => {
                self.lobby.play_pending = true;

                // if the server wants us to load the map and we arent already doing that, do it
                if self.beatmap_loader.is_none() {

                    // only load map if we have it selected
                    if self.current_beatmap_is_selected() {
                        let Some(mode) = self.selected_mode.clone()
                        else { return Ok(None) };
                        let Some(map) = values.beatmap_manager.current_beatmap()
                        else { return Ok(None) };

                        let mods = values.global.mods.clone();
                        let infos = self.infos.clone();
                        let map = map.clone();
                        let settings = values.settings.clone();
                        let f =
                        // let f = async move {
                            GameplayManager::create(
                                &infos,
                                &mode,
                                &map,
                                None,
                                mods,
                                &settings,
                                database,
                            );
                        // };
                        self.beatmap_loader = Some(f);
                        // self.beatmap_loader = Some(engine::io::AsyncLoader::new(f));
                    } else {
                        error!("not loading map: current != selected");
                    }
                }
            }
            MultiplayerPacket::Server_LobbyBeginRound => {
                self.lobby.should_play = true;

                // if we're not playing yet
                if manager.is_none() {
                    let mut new_manager = None;

                    if let Some(loader) = self.beatmap_loader.take() {
                        if let Some(manager) = Some(loader) { // .check() {
                            match manager {
                                Ok(mut manager) => {
                                    manager.set_mode(actions::game::GameplayTypeInfo::Multiplayer.into());
                                    new_manager = Some(manager);
                                    self.set_state(LobbyUserState::InGame, actions);
                                }
                                Err(e) => error!("no manager! {e:?}"),
                            }

                        }
                    } else {
                        warn!("no loader!!!");
                    }
                    self.beatmap_loader = None;
                    self.load_complete_sent = false;

                    self.lobby.play_pending = false;
                    self.lobby.should_play = false;

                    return Ok(new_manager);
                }
            }

            MultiplayerPacket::Server_LobbyMapChange {
                lobby_id,
                new_map
            } => {
                if &self.lobby.id != lobby_id { return Ok(None) };
                self.lobby.info.current_beatmap = Some(new_map.clone());
                warn!("lobby map change to {new_map:?}");

                if let Some(beatmap) = &self.lobby.current_beatmap {
                    // update the playmode
                    self.selected_mode = Some(beatmap.mode.clone().into());
                    actions.push(BeatmapAction::SetPlaymode(beatmap.mode.clone()).into());

                    // the beatmap change handler in Self::update will handle the rest
                    actions.push(BeatmapAction::Set(
                        beatmap.hash,
                        SetBeatmapOptions::default().restart_song(true)
                    ).into());
                } else {
                    actions.push(BeatmapAction::Remove.into());
                    self.set_state(LobbyUserState::NoMap, actions);
                }
            }

            MultiplayerPacket::Server_LobbyModsChanged { free_mods, mods, speed } => {
                // let mut current_lobby = CurrentLobbyInfo::get_mut();
                // let Some(lobby) = &mut *current_lobby else { continue };
                // lobby.free_mods = free_mods
                // lobby.mods = mods;
                // lobby.speed = speed;


                if !free_mods {
                    actions.push(ModAction::SetMods(mods.clone()).into());
                    // values.global.mods.mods = mods.clone();
                    // values.global.mods.set_speed(*speed);
                }
                // TODO: do we want to force the speed even with free mods?()
                actions.push(ModAction::SetSpeed(GameSpeed::from_u8(*speed)).into());
                // values.global.mods.set_speed(*speed);
            }

            MultiplayerPacket::Server_LobbyUserModsChanged { user_id, mods, speed } => {
                if let Some(u) = self.lobby.players
                    .iter_mut()
                    .find(|u| &u.user_id == user_id)
                {
                    u.mods = mods.clone();
                    u.speed = *speed;
                };
            }

            MultiplayerPacket::Server_LobbyPlayerMapComplete { user_id, score } => {
                self.lobby.player_scores.insert(*user_id, score.clone());
            }

            MultiplayerPacket::Server_LobbyRoundComplete => {
                info!("lobby round completed");
                #[cfg(feature="graphics")]
                actions.push(MenuAction::set_menu("score_menu").into());
            }

            MultiplayerPacket::Server_LobbyScoreUpdate { user_id, score } => {
                self.lobby.player_scores.insert(*user_id, score.clone());

                // update the manager
                if let Some(manager) = manager {
                    manager.score_list.scores = self.lobby.player_scores.iter()
                        .filter(|(u,_)| u != &&self.lobby.our_user_id) // make sure we dont re-add our own score in
                        .map(|(_,s)| IngameScore::new(
                            s.clone(),
                            false,
                            false
                        ))
                        .collect();

                    manager
                        .score_list
                        .scores
                        .sort_by(|a, b| b.score.score.cmp(&a.score.score));
                }
            }

            MultiplayerPacket::Server_LobbyStateChange {
                lobby_id,
                new_state
            } => {
                if lobby_id != &self.lobby.id { return Ok(None) }
                self.lobby.info.state = *new_state;
            }

            MultiplayerPacket::Server_LobbyChangeHost { new_host } => {
                let was_host = self.is_host();
                self.lobby.host = *new_host;

                // if we just became the host, show a notif
                if self.is_host() && !was_host {
                    actions.push(
                        Notification::default()
                        .text("You are now the host!")
                        .duration(3000.0)
                        .color(Color::PURPLE_AMETHYST)
                        .into()
                    );
                }
            }
            // MultiplayerPacket::Server_SkipRequest => {
            //     self.skip_request_sent = false;
            //     if let Some(manager) = manager {
            //         manager.skip_intro();
            //     }
            // }

            _ => {}
        }

        self.update_values(values);

        Ok(None)
    }

    fn send_packet(
        &mut self,
        packet: impl Into<PacketId>,
        actions: &mut actions::ActionQueue,
    ) {
        actions.push(OnlineAction::Packet(Box::new(packet.into())).into());
    }
    fn set_state(
        &mut self,
        new_state: LobbyUserState,
        actions: &mut actions::ActionQueue,
    ) {
        self.send_packet(
            MultiplayerPacket::Client_LobbyUserState { new_state },
            actions,
        );
    }

    pub fn handle_lobby_action(
        &mut self,
        action: LobbyAction,
        settings: &engine::Settings,
        actions: &mut actions::ActionQueue,
    ) {
        match action {
            LobbyAction::Start => {
                self.skip_request_sent = false;
                self.send_packet(MultiplayerPacket::Client_LobbyStart, actions);
            }

            LobbyAction::Ready => {
                self.set_state(LobbyUserState::Ready, actions);
            }
            LobbyAction::Unready => {
                self.set_state(LobbyUserState::NotReady, actions);
            }

            LobbyAction::SendSkipRequest => {
                self.skip_request_sent = true;
            }

            LobbyAction::MapComplete(mut score) => {
                // dont include the replay for this message
                score.replay = None;
                self.send_packet(
                    MultiplayerPacket::Client_LobbyMapComplete {
                        score: *score
                    },
                    actions,
                );
            }

            LobbyAction::OpenMapLink => {
                let Some(beatmap) = &self.lobby.current_beatmap
                else { return };

                let hash = beatmap.hash;
                let score_url = settings.connection().score_url.clone();


                // TODO: maybe move to a task?
                // or maybe readd direct????
                let req = ureq::get(
                    format!("{score_url}/api/get_beatmap_url?hash={hash}")
                ).call();
                match req {
                    Err(e) => actions.push(Notification::new_error(
                        "Error with beatmap url request",
                        e.to_string()
                    ).into()),

                    Ok(resp) => {
                        #[allow(unused)] #[derive(Deserialize)]
                        struct Resp { error: Option<String>, url: Option<String> }

                        let Ok(body) = resp.into_body().read_to_string() else {
                            actions.push(
                                Notification::default()
                                .text("shit")
                                .duration(3000.0)
                                .color(Color::RED)
                                .into()
                            );
                            return;
                        };
                        info!("url resp: {body}");

                        match serde_json::from_str(&body) {
                            Ok(Resp {url: Some(url), ..}) => tataku::open_link(url),
                            _ => error!("some shit broke i dont care")
                        }
                    }
                }
            }

            // slot actions
            LobbyAction::SlotAction(LobbySlotAction::ShowProfile(slot)) => {
                let Some(LobbySlot::Filled { user: _ }) = self.lobby.slots.get(&slot)
                else { return };

                // TODO: set values for user_id and slot_id, then open dialog
                // self.actions.push()
            }

            LobbyAction::SlotAction(LobbySlotAction::MoveTo(slot)) => {
                let Some(LobbySlot::Empty) = self.lobby.slots.get(&slot)
                else { return };

                let user = self.lobby.our_user_id;
                self.send_packet(
                    MultiplayerPacket::Client_LobbySlotChange {
                        slot,
                        new_status: LobbySlot::Filled { user }
                    },
                    actions
                );
            }

            LobbyAction::SlotAction(LobbySlotAction::TransferHost(slot)) => {
                if !self.is_host() { return }
                let Some(LobbySlot::Filled { user }) = self.lobby.slots.get(&slot)
                else { return };

                self.send_packet(
                    MultiplayerPacket::Client_LobbyChangeHost {
                        new_host: *user
                    },
                    actions,
                );
            }

            LobbyAction::SlotAction(LobbySlotAction::Lock(slot))
            | LobbyAction::SlotAction(LobbySlotAction::Kick(slot))
            => {
                if !self.is_host() { return }
                self.send_packet(
                    MultiplayerPacket::Client_LobbySlotChange {
                        slot,
                        new_status: LobbySlot::Locked
                    },
                    actions
                );
            }
            LobbyAction::SlotAction(LobbySlotAction::Unlock(slot)) => {
                if !self.is_host() { return }
                self.send_packet(
                    MultiplayerPacket::Client_LobbySlotChange {
                        slot,
                        new_status: LobbySlot::Empty
                    },
                    actions
                );
            }

            _ => {}
        }
    }

    pub fn is_host(&self) -> bool {
        self.lobby.is_host()
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



#[test]
fn test() {
    use crate::prelude::*;

    #[derive(Reflect, Debug, Clone)]
    #[reflect(display="debug")]
    struct A { a: Option<B> }
    #[derive(Reflect, Debug, Clone)]
    #[reflect(display="debug")]
    struct B { b: u32 }


    let a = A {
        a: Some(B { b: 12 })
    };
    let a: Box<dyn Reflect> = Box::new(a);

    let b = a.reflect_get::<u32>("a.b").unwrap();
    println!("{}", *b);
}
