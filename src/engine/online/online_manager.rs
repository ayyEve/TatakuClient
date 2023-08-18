use PacketId::*;
use tataku_common::*;
use crate::prelude::*;
use super::online_user::OnlineUser;
use tokio::{ sync::Mutex, net::TcpStream };
use futures_util::{ SinkExt, StreamExt, stream::SplitSink };
use tokio_tungstenite::{ MaybeTlsStream, WebSocketStream, tungstenite::protocol::Message };

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;


// how many frames do we buffer before sending?
// higher means less packet spam
const SPECTATOR_BUFFER_FLUSH_SIZE: usize = 20;
type ThreadSafeSelf = Arc<AsyncRwLock<OnlineManager>>;

lazy_static::lazy_static! {
    static ref ONLINE_MANAGER:ThreadSafeSelf = Arc::new(AsyncRwLock::new(OnlineManager::new()));
}
///
pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub friends: HashSet<u32>, // userid is key
    pub discord: Option<Discord>,

    pub user_id: u32, // this user's id
    /// are we successfully logged in?
    pub logged_in: bool,

    /// socket writer
    pub writer: Option<Arc<Mutex<WsWriter>>>,

    // ====== chat ======
    pub chat_messages: HashMap<ChatChannel, Vec<ChatMessage>>,

    // ====== spectator ======
    pub spectator_info: OnlineSpectatorInfo,
}

impl OnlineManager {
    pub fn new() -> OnlineManager {
        let mut messages = HashMap::new();
        let channel = ChatChannel::Channel{name: "general".to_owned()};
        messages.insert(channel.clone(), vec![ChatMessage::new(
            "System".to_owned(),
            channel,
            u32::MAX,
            "this is a test message".to_owned()
        )]);

        OnlineManager {
            user_id: 0,
            logged_in: false,
            users: HashMap::new(),
            friends: HashSet::new(),
            discord: None,
            // chat: Chat::new(),
            writer: None,
            connected: false,
            chat_messages: messages,
            spectator_info: OnlineSpectatorInfo::new(0),
        }
    }

    pub async fn init_discord() {
        match Discord::new() {
            Ok(discord) => OnlineManager::get_mut().await.discord = Some(discord),
            Err(e) => error!("discord error: {e}"),
        }
    }

    pub async fn start() {
        info!("starting network connection");
        let mut settings = SettingsHelper::new();

        // insert multiplayer data
        GlobalValueManager::update(Arc::new(MultiplayerData::default()));
        GlobalValueManager::update::<Option<CurrentLobbyInfo>>(Arc::new(None));

        let server_url = settings.server_url.clone();

        // initialize the connection
        match tokio_tungstenite::connect_async(settings.server_url.clone()).await {
            Ok((ws_stream, _)) => {
                let (writer, mut reader) = ws_stream.split();
                let writer = Arc::new(Mutex::new(writer));

                // send login
                {
                    let mut s = OnlineManager::get_mut().await;
                    s.writer = Some(writer);
                    s.connected = true;

                    // send login packet
                    s.send_packet(Client_UserLogin {
                        protocol_version: 1,
                        game: "Tataku\n0.1.0".to_owned(),
                        username: settings.username.clone(),
                        password: settings.password.clone()
                    }).await;
                }

                while let Some(message) = reader.next().await {
                    if settings.update() {
                        if settings.server_url != server_url {
                            info!("server url changed, restarting network manager");
                            Self::restart();
                            return;
                        }
                    }

                    match message {
                        Ok(Message::Binary(data)) => {
                            if let Err(e) = OnlineManager::handle_packet(data, &settings.logging_settings).await {
                                error!("Error with packet: {}", e);
                            }
                        }
                        Ok(Message::Ping(_)) => {
                            if let Some(writer) = &OnlineManager::get().await.writer {
                                let _ = writer.lock().await.send(Message::Pong(Vec::new())).await;
                            }
                        }

                        Ok(Message::Close(_)) => {
                            NotificationManager::add_text_notification("Disconnected from server", 5000.0, Color::RED).await;
                            Self::disconnect().await;
                        }
                        Ok(message) => if settings.logging_settings.extra_online_logging { warn!("Got other network message: {message:?}"); },

                        Err(oof) => {
                            error!("network connection error: {oof}\nAttempting to reconnect");
                            Self::disconnect().await;

                            // reconnect handled by caller
                            break;
                        }
                    }
                }
            }
            Err(oof) => {
                // s.write().await.connected = false;
                warn!("Could not accept connection: {}", oof);
            }
        }
    }

    /// disconnect and reset everything
    pub fn restart() {
        tokio::spawn(async {
            OnlineManager::get_mut().await.reset().await;

            // // re-establish the connection
            // Self::start().await;
        });
    }

    /// disconnect and reset everything
    async fn reset(&mut self) {
        // if we currently have a connection, close it
        if let Some(writer) = &self.writer {
            let _ = writer.lock().await.close();
        }

        // reset most values
        self.writer = None;
        self.user_id = 0;
        self.connected = false;
        self.logged_in = false;
        self.users.clear();
        self.friends.clear();

        self.spectator_info = OnlineSpectatorInfo::default();
        
        MultiplayerData::get_mut().clear();
        *CurrentLobbyInfo::get_mut() = None;
    }

    /// disconnect without resetting anything
    async fn disconnect() {
        let mut s = OnlineManager::get_mut().await;
        s.writer = None;
        s.connected = false;
        s.logged_in = false;
    }

    /// handle an incoming server packet
    async fn handle_packet(data:Vec<u8>, log_settings: &LoggingSettings) -> TatakuResult<()> {
        let mut reader = SerializationReader::new(data);

        while reader.can_read() {
            let packet:PacketId = reader.read()?;
            if log_settings.extra_online_logging { info!("Got packet {:?}", packet); };

            match packet {
                // ===== ping/pong =====
                PacketId::Ping => { OnlineManager::get().await.send_packet(Pong).await; },
                PacketId::Pong => {/* trace!("Got pong from server"); */},

                // login
                PacketId::Server_LoginResponse { status, user_id } => {
                    match status {
                        LoginStatus::UnknownError => {
                            trace!("Unknown Error");
                            NotificationManager::add_text_notification("[Login] Unknown error logging in", 5000.0, Color::RED).await;
                        }
                        LoginStatus::BadPassword => {
                            trace!("Auth failed");
                            NotificationManager::add_text_notification("[Login] Authentication failed", 5000.0, Color::RED).await;
                        }
                        LoginStatus::NoUser => {
                            trace!("User not found");
                            NotificationManager::add_text_notification("[Login] Authentication failed", 5000.0, Color::RED).await;
                        }
                        LoginStatus::Ok => {
                            trace!("Success, got user_id: {}", user_id);
                            {
                                let mut om = OnlineManager::get_mut().await;
                                om.user_id = user_id;
                                om.logged_in = true;
                                om.spectator_info = OnlineSpectatorInfo::new(user_id);
                            }
                            NotificationManager::add_text_notification("[Login] Logged in!", 2000.0, Color::GREEN).await;

                            ping_handler();

                            // request friends list
                            OnlineManager::get().await.send_packet(ChatPacket::Client_GetFriends).await;
                        }
                    }
                }

                // notification
                PacketId::Server_Notification { message, severity } => {
                    let (color, duration) = match severity {
                        Severity::Info => (Color::GREEN, 3000.0),
                        Severity::Warning => (Color::YELLOW, 5000.0),
                        Severity::Error => (Color::RED, 7000.0),
                    };

                    NotificationManager::add_text_notification(&message, duration, color).await;
                }
                // server error
                PacketId::Server_Error { code, error } => {
                    warn!("Got server error {:?}: '{}'", code, error)
                }


                // ===== user updates =====
                PacketId::Server_UserJoined { user_id, username, game } => {
                    if log_settings.extra_online_logging { debug!("User {} joined (id: {}, game: {})", username, user_id, game); };
                    let mut user = OnlineUser::new(user_id, username.clone());
                    user.game = game;

                    let mut s = OnlineManager::get_mut().await;
                    s.users.insert(user_id, Arc::new(Mutex::new(user)));

                    // if this is us, make sure we have the correct username text
                    if s.user_id == user_id {
                        let mut settings = Settings::get_mut();
                        if settings.username != username {
                            settings.username = username.clone()
                        }
                    }

                    if s.friends.contains(&user_id) {
                        NotificationManager::add_text_notification(format!("{username} is online"), 5000.0, Color::BLUE).await;
                    }
                }
                PacketId::Server_UserLeft {user_id} => {
                    if log_settings.extra_online_logging { debug!("User id {user_id} left"); };

                    let mut lock = OnlineManager::get_mut().await;
                    // remove from online users
                    if let Some(u) = lock.users.remove(&user_id) {
                        let l = u.lock().await;
                        let username = &l.username;
                        if lock.friends.contains(&user_id) {
                            NotificationManager::add_text_notification(format!("{username} is offline"), 5000.0, Color::BLUE).await;
                        }
                    }

                    // remove from our spec list
                    lock.spectator_info.remove_spec(0, user_id);
                }
                PacketId::Server_UserStatusUpdate { user_id, action, action_text, mode } => {
                    // debug!("Got user status update: {}, {:?}, {} ({:?})", user_id, action, action_text, mode);
                    
                    if let Some(e) = OnlineManager::get().await.users.get(&user_id) {
                        let mut a = e.lock().await;
                        a.action = Some(action);
                        a.action_text = Some(action_text);
                        a.mode = Some(mode);
                    }
                }

                // score 
                PacketId::Server_ScoreUpdate { .. } => {}

                // ===== chat =====
                PacketId::Chat_Packet { packet } => Self::handle_chat_packet(packet, log_settings).await?,
                
                // ===== spectator =====
                PacketId::Spectator_Packet { host_id, packet } => Self::handle_spec_packet(packet, host_id, log_settings).await?,
                
                // ===== multiplayer =====
                PacketId::Multiplayer_Packet { packet } => Self::handle_multi_packet(packet, log_settings).await?,

                // other packets
                PacketId::Unknown => {
                    warn!("Got unknown packet, dropping remaining packets");
                    break;
                }

                p => {
                    warn!("Got unhandled packet: {p:?}, dropping remaining packets");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_chat_packet(packet: ChatPacket, log_settings: &LoggingSettings) -> TatakuResult<()> {
        match packet {
            ChatPacket::Server_SendMessage {sender_id, message, channel}=> {
                if log_settings.extra_online_logging { debug!("Got message: `{}` from user id `{}` in channel `{}`", message, sender_id, channel); };

                let channel = if channel.starts_with("#") {
                    ChatChannel::Channel {name: channel.trim_start_matches("#").to_owned()}
                } else {
                    ChatChannel::User {username: channel}
                };

                let mut lock = OnlineManager::get_mut().await;
                let sender = lock.find_user_by_id(sender_id).unwrap_or_default().lock().await.username.clone();
                let chat_messages = &mut lock.chat_messages;
                // if the list doesnt include the channel, add it
                if !chat_messages.contains_key(&channel) {
                    chat_messages.insert(channel.clone(), Vec::new());
                }

                let message = ChatMessage::new(
                    sender,
                    channel.clone(),
                    sender_id,
                    message
                );

                // add the message to the channel
                chat_messages.get_mut(&channel).unwrap().push(message);
            }

            // friends list received from server
            ChatPacket::Server_FriendsList { friend_ids } => {
                let mut s = OnlineManager::get_mut().await;
                for i in friend_ids.iter() {
                    if let Some(u) = s.users.get_mut(i) {
                        u.lock().await.friend = true;
                    }
                }
                info!("got friends list: {friend_ids:?}");

                s.friends = friend_ids.into_iter().collect();
            }

            ChatPacket::Server_UpdateFriend { friend_id, is_friend } => {
                let mut s = OnlineManager::get_mut().await;
                if let Some(u) = s.users.get_mut(&friend_id) {
                    u.lock().await.friend = is_friend;
                }

                if is_friend {
                    s.friends.insert(friend_id);
                    info!("add friend {friend_id}");
                } else {
                    s.friends.remove(&friend_id);
                    info!("remove friend {friend_id}");
                }
            }

            _ => {}
        }

        Ok(())
    }

    async fn handle_spec_packet(packet: SpectatorPacket, host_id: u32, _log_settings: &LoggingSettings) -> TatakuResult<()> {
        match packet {
            SpectatorPacket::Server_SpectatorFrames { frames: new_frames } => {
                // debug!("Got {} spectator frames from the server", frames.len());
                let mut lock = OnlineManager::get_mut().await;
                if let Some(frames) = lock.spectator_info.incoming_frames.get_mut(&host_id) {
                    frames.extend(new_frames);
                } else {
                    warn!("got spec packets for host we're not spectating: {host_id}");
                }
            }
            // spec join/leave
            SpectatorPacket::Server_SpectatorJoined { user_id, username }=> {
                OnlineManager::get_mut().await.spectator_info.add_spec(host_id, user_id, username.clone());
                NotificationManager::add_text_notification(format!("{username} is now spectating"), 2000.0, Color::GREEN).await;
            }
            SpectatorPacket::Server_SpectatorLeft { user_id } => {
                let user = if let Some(u) = OnlineManager::get().await.find_user_by_id(user_id) {
                    u.lock().await.username.clone()
                } else {
                    "A user".to_owned()
                };
                OnlineManager::get_mut().await.spectator_info.remove_spec(host_id, user_id);
                
                NotificationManager::add_text_notification(format!("{user} stopped spectating"), 2000.0, Color::GREEN).await;
            }
            SpectatorPacket::Server_SpectateResult { result} => {
                trace!("Got spec result {result:?}");
                match result {
                    SpectateResult::Ok => OnlineManager::get_mut().await.spectator_info.add_host(host_id),
                    SpectateResult::Error_SpectatingBot => NotificationManager::add_text_notification("You cannot spectate a bot!", 3000.0, Color::RED).await,
                    SpectateResult::Error_HostOffline => NotificationManager::add_text_notification("Spectate host is offline!", 3000.0, Color::RED).await,
                    SpectateResult::Error_SpectatingYourself => NotificationManager::add_text_notification("You cannot spectate yourself!", 3000.0, Color::RED).await,
                    SpectateResult::Error_Unknown => NotificationManager::add_text_notification("Unknown error trying to spectate!", 3000.0, Color::RED).await,
                }
            }

            _ => {}
        }

        Ok(())
    }

    async fn handle_multi_packet(packet: MultiplayerPacket, _log_settings: &LoggingSettings) -> TatakuResult<()> {
        match packet {
            MultiplayerPacket::Server_LobbyList { lobbies } => {
                let mut multi_data = MultiplayerData::get_mut();
                // let mut multi_data = multi_data.write().await;
                multi_data.lobbies = lobbies.into_iter().map(|l|(l.id, l)).collect();
            }

            MultiplayerPacket::Server_CreateLobby { success, lobby } => {
                let multi_data = MultiplayerData::get();
                if success && multi_data.lobby_creation_pending {
                    let our_id = OnlineManager::get().await.user_id;
                    
                    let mut current_lobby = CurrentLobbyInfo::get_mut();
                    *current_lobby = lobby.map(|i|CurrentLobbyInfo::new(i, our_id));
                    
                    if let Some(current) = &mut *current_lobby {
                        current.update_usernames().await;
                    }

                    // should update the server with our current map and mode
                    if let Some(map) = CurrentBeatmapHelper::new().0.clone() {
                        let mode = CurrentPlaymodeHelper::new().0.clone();
                        Self::update_lobby_beatmap(map, mode).await;
                    }

                }
            }
            MultiplayerPacket::Server_JoinLobby { success, lobby } => {
                let multi_data: Arc<MultiplayerData> = MultiplayerData::get();
                if success && multi_data.lobby_join_pending {
                    let our_id = OnlineManager::get().await.user_id;
                    
                    let mut current_lobby = CurrentLobbyInfo::get_mut();
                    *current_lobby = lobby.map(|i|CurrentLobbyInfo::new(i, our_id));
                    if let Some(current) = &mut *current_lobby {
                        current.update_usernames().await;
                    } else {
                        info!("didnt set lobby?")
                    }
                }
            }


            MultiplayerPacket::Server_LobbyCreated { lobby } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.insert(lobby.id, lobby);
            }
            MultiplayerPacket::Server_LobbyDeleted { lobby_id } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.remove(&lobby_id);
            }

            MultiplayerPacket::Server_LobbyUserJoined { lobby_id, user_id } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.get_mut(&lobby_id).ok_do_mut(|l|l.players.push(user_id));

                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(our_lobby) = &mut *current_lobby else { return Ok(()) };
                if our_lobby.info.id == lobby_id {
                    our_lobby.info.players.push(LobbyUser { user_id, ..Default::default() });

                    let Some(user) = OnlineManager::get().await.users.get(&user_id).cloned() else { 
                        NotificationManager::add_text_notification(format!("user with id {} joined the match", user_id), 3000.0, Color::PURPLE).await;
                        return Ok(())
                    };
                    let user = user.lock().await;
                    our_lobby.player_usernames.insert(user_id, user.username.clone());
                    
                    NotificationManager::add_text_notification(format!("{} joined the match", user.username), 3000.0, Color::PURPLE).await;
                }
            }
            MultiplayerPacket::Server_LobbyUserLeft { lobby_id, user_id } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.get_mut(&lobby_id).map(|l|l.players.retain(|u|u != &user_id));


                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(our_lobby) = &mut *current_lobby else { return Ok(()) };
                if our_lobby.id != lobby_id { return Ok(()); }

                our_lobby.players.retain(|u|u.user_id != user_id);

                // find the slot that had this user and set it to empty (server will update its proper status next update)
                our_lobby.slots.values_mut().find(|s|**s == LobbySlot::Filled{user: user_id}).ok_do_mut(|s|**s = LobbySlot::Empty);
                
                
                if user_id == our_lobby.our_user_id {
                    tokio::spawn(async {*CurrentLobbyInfo::get_mut() = None});
                    NotificationManager::add_text_notification("You have been kicked from the match", 3000.0, Color::PURPLE).await;
                } else {
                    let username = our_lobby.player_usernames.remove(&user_id).unwrap_or_default();
                    NotificationManager::add_text_notification(format!("{username} left the match"), 3000.0, Color::PURPLE).await;
                }
            }

            MultiplayerPacket::Server_LobbySlotChange { slot, new_status } => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                lobby.info.slots.get_mut(&slot).ok_do_mut(|s|**s = new_status);
            }

            MultiplayerPacket::Server_LobbyUserState { user_id, new_state } => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                lobby.info.players.iter_mut().find(|u|u.user_id == user_id).ok_do_mut(|u|u.state = new_state);
            }


            MultiplayerPacket::Server_LobbyStart => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                lobby.play_pending = true;
            }
            MultiplayerPacket::Server_LobbyBeginRound => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                lobby.should_play = true;
            }

            MultiplayerPacket::Server_LobbyMapChange { lobby_id, new_map } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.get_mut(&lobby_id).ok_do_mut(|l|l.current_beatmap = Some(new_map.title.clone()));
            

                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                if lobby.id == lobby_id {
                    lobby.info.current_beatmap = Some(new_map);
                }
            }

            //TODO: implement no free mods
            MultiplayerPacket::Server_LobbyModsChanged { free_mods:_, mods:_, speed:_ } => {
                // let mut current_lobby = CurrentLobbyInfo::get_mut();
                // let Some(lobby) = &mut *current_lobby else { continue };
                // lobby.free_mods = free_mods
                // lobby.mods = mods;
                // lobby.speed = speed;
            }

            MultiplayerPacket::Server_LobbyUserModsChanged { user_id, mods, speed } => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };

                lobby.players.iter_mut().find(|u|u.user_id == user_id).ok_do_mut(|u| {
                    u.mods = mods;
                    u.speed = speed;
                });
            }

            MultiplayerPacket::Server_LobbyPlayerMapComplete { user_id, score } => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                lobby.player_scores.insert(user_id, score);
            }

            MultiplayerPacket::Server_LobbyRoundComplete => {
                info!("lobby round completed");
            }

            MultiplayerPacket::Server_LobbyScoreUpdate { user_id, score } => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                let Some(lobby) = &mut *current_lobby else { return Ok(()) };
                lobby.player_scores.insert(user_id, score);
            }
            
            MultiplayerPacket::Server_LobbyStateChange { lobby_id, new_state } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.get_mut(&lobby_id).ok_do_mut(|l|l.state = new_state);
                
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                current_lobby.as_mut().filter(|l|l.info.id == lobby_id).ok_do_mut(|l|l.info.state = new_state);
            }

            MultiplayerPacket::Server_LobbyInvite { inviter_id, lobby } => {
                let mut multi_data = MultiplayerData::get_mut();
                multi_data.lobbies.get_mut(&lobby.id).ok_do_mut(|l|l.has_password = false);

                let Some(inviter) = OnlineManager::get().await.users.get(&inviter_id).cloned() else { return Ok(()) };
                let inviter = inviter.lock().await;
                let text = format!("{} has invited you to a multiplayer match", inviter.username);

                let notif = Notification::new(text, Color::PURPLE_AMETHYST, 10_000.0, NotificationOnClick::MultiplayerLobby(lobby.id));
                NotificationManager::add_notification(notif).await;
            }

            MultiplayerPacket::Server_LobbyChangeHost { new_host } => {
                let mut current_lobby = CurrentLobbyInfo::get_mut();
                current_lobby.ok_do_mut(|l|l.info.host = new_host);
            }

            _ => {}
        }


        Ok(())
    }



    /// set our user's action for the server and any enabled integrations
    pub fn set_action(action_info: SetAction, incoming_mode: Option<String>) {
        tokio::spawn(async move {
            let s = OnlineManager::get().await;
            let mode = incoming_mode.clone().unwrap_or(String::new());


            let action = action_info.get_action();
            let action_text = match &action_info {
                SetAction::Idle => format!("Idle"),
                SetAction::Closing => format!("Closing"),

                SetAction::Listening { artist, title, .. } => format!("Listening to {artist} - {title}"),
                SetAction::Spectating { player, artist, title, version, creator:_ } => format!("Watching {player} play {artist} - {title}[{version}]"),
                SetAction::Playing { artist, title, version, .. } => format!("Playing {artist} - {title}[{version}]"),
            };


            s.send_packet(PacketId::Client_StatusUpdate { action, action_text: action_text.clone(), mode }).await;
            if action == UserAction::Leaving {
                s.send_packet(PacketId::Client_LogOut).await;
            }


            if let Some(discord) = &s.discord {
                discord.change_status(&action_info, incoming_mode).await;
            }

            if Settings::get().integrations.lastfm {
                match &action_info {
                    SetAction::Listening { artist, title, .. } 
                    | SetAction::Playing { artist, title, .. } 
                    | SetAction::Spectating { artist, title, .. } => {
                        LastFmIntegration::update(title.clone(), artist.clone()).await;
                    }
                    _ => {}
                }
            }

        });
    }

    pub fn find_user_by_id(&self, user_id: u32) -> Option<Arc<Mutex<OnlineUser>>> {
        self.users.get(&user_id).cloned()
    }
}

// spectator functions
impl OnlineManager {
    pub fn send_spec_frames(frames:Vec<SpectatorFrame>, force_send: bool) {
        tokio::spawn(async move {
            let mut lock = OnlineManager::get_mut().await;
            // if we arent speccing, exit
            // hopefully resolves a bug
            // if !lock.spectating {return}

            lock.spectator_info.outgoing_frames.extend(frames);
            // wait at most 1s before sending packets
            let times_up = lock.spectator_info.last_sent_frame.as_millis() > 1000.0;

            if force_send || times_up || lock.spectator_info.outgoing_frames.len() >= SPECTATOR_BUFFER_FLUSH_SIZE {
                let frames = std::mem::take(&mut lock.spectator_info.outgoing_frames);
                
                // info!("Sending {} spec packets", frames.len());
                lock.send_packet(SpectatorPacket::Client_SpectatorFrames {frames}.with_host(lock.user_id)).await;
                lock.spectator_info.last_sent_frame = Instant::now();
            }
        });

    }

    /// attempt to start spectating a host
    /// 
    /// this doesnt actually begin the spec process, it just sends the request to the server
    /// the process starts when the spec is approved
    pub fn start_spectating(host_id: u32) {
        tokio::spawn(async move {
            let s = OnlineManager::get().await;
            s.send_packet(SpectatorPacket::Client_Spectate.with_host(host_id)).await;
        });
    }

    pub fn stop_spectating(host_id: u32) {
        info!("Request to stop speccing {host_id}");
        tokio::spawn(async move {
            trace!("Attempting to stop speccing {host_id}");
            let mut s = OnlineManager::get_mut().await;
            s.spectator_info.remove_host(host_id);
            s.send_packet(SpectatorPacket::Client_LeaveSpectator.with_host(host_id)).await;
            trace!("Stopped speccing {host_id}");
        });
    }

    pub fn get_pending_spec_frames(&mut self, host_id: u32) -> Vec<SpectatorFrame> {
        let Some(frames) = self.spectator_info.incoming_frames.get_mut(&host_id) else { return Vec::new() };
        std::mem::take(frames)
    }
}

// multiplayer functions
impl OnlineManager {
    pub async fn add_lobby_listener() {
        // info!("add lobby listener");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_AddLobbyListener).await;
        s.send_packet(MultiplayerPacket::Client_LobbyList).await;
    }
    pub async fn remove_lobby_listener() {
        // info!("remove lobby listener");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_AddLobbyListener).await;
    }

    pub async fn create_lobby(name: String, password: String, private: bool, players: u8) {
        // info!("create lobby");
        let s = OnlineManager::get().await;
        MultiplayerData::get_mut().lobby_creation_pending = true;
        s.send_packet(MultiplayerPacket::Client_CreateLobby { name, password, private, players }).await;
    }

    pub async fn join_lobby(lobby_id: u32, password: String) {
        // if we're already in a lobby
        if let Some(lobby) = &*CurrentLobbyInfo::get() { 
            // and its the lobby we want to join, dont do anything
            if lobby.id == lobby_id { return }
            // otherwise, leave the current lobby
            Self::leave_lobby().await; 
        }
        // info!("join lobby");

        let s = OnlineManager::get().await;
        MultiplayerData::get_mut().lobby_join_pending = true;
        s.send_packet(MultiplayerPacket::Client_JoinLobby { lobby_id, password }).await;
    }

    pub async fn leave_lobby() {
        // info!("leaving lobby");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LeaveLobby).await;
        *CurrentLobbyInfo::get_mut() = None;
    }

    pub async fn invite_user(user_id: u32) {
        // info!("inviting user {user_id}");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyInvite { user_id } ).await;
    }

    pub async fn update_lobby_beatmap(beatmap: Arc<BeatmapMeta>, mode: String) {
        // info!("update lobby beatmap: {beatmap:?}, {mode}");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyMapChange { 
            new_map: LobbyBeatmap { 
                title: beatmap.version_string(), 
                hash: beatmap.beatmap_hash.clone(), 
                mode,
                map_game: beatmap.beatmap_type.into()
            }
        }).await;
    }

    pub async fn move_lobby_slot(new_slot: u8) {
        // info!("move slot");
        let s = OnlineManager::get().await;
        let our_id = s.user_id;
        s.send_packet(MultiplayerPacket::Client_LobbySlotChange { slot: new_slot, new_status: LobbySlot::Filled {user: our_id} } ).await;
    }
    pub async fn update_lobby_slot(slot: u8, new_state: LobbySlot) {
        // info!("update slot");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbySlotChange { slot, new_status: new_state } ).await;
    }

    pub async fn update_lobby_state(new_state: LobbyUserState) {
        // info!("update our user state: {new_state:?}");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyUserState { new_state } ).await;
    }

    pub async fn lobby_load_complete() {
        // info!("sending load complete");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyMapLoaded).await;
    }

    pub async fn lobby_map_complete(score: Score) {
        // info!("sending map complete");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyMapComplete { score }).await;
    }

    pub async fn lobby_map_start() {
        // info!("sending map start");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyStart).await;
    }

    pub async fn lobby_update_score(score: Score) {
        // info!("update lobby score");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyScoreUpdate { score }).await;
    }

    pub async fn lobby_change_host(new_host: u32) {
        // info!("change lobby host");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyChangeHost { new_host }).await;
    }

    pub async fn lobby_kick_user(user: u32) {
        // find the user's slot, and set it to locked (kick action)
        let Some(this_lobby) = &*CurrentLobbyInfo::get() else { return };
        let Some((slot, _)) = this_lobby.slots.iter().find(|(_,s)|s == &&LobbySlot::Filled { user }) else { return };
        Self::update_lobby_slot(*slot, LobbySlot::Locked).await;
    }

    pub async fn lobby_update_mods(mods: HashSet<String>, speed: u16) {
        // info!("update mods and speed");
        let s = OnlineManager::get().await;
        s.send_packet(MultiplayerPacket::Client_LobbyUserModsChanged { mods, speed }).await;
    }
}

impl OnlineManager {
    /// opens a read lock on the online manager
    pub async fn get<'a>() -> tokio::sync::RwLockReadGuard<'a, Self> {
        ONLINE_MANAGER.read().await
    }
    /// opens a write lock on the online manager
    pub async fn get_mut<'a>() -> tokio::sync::RwLockWriteGuard<'a, Self> {
        ONLINE_MANAGER.write().await
    }
    /// try to open a read lock on the online manager, returning None if the lock was unsuccessful
    pub fn try_get<'a>() -> Option<tokio::sync::RwLockReadGuard<'a, Self>> {
        ONLINE_MANAGER.try_read().ok()
    }
    /// try to open a write lock on the online manager, returning None if the lock was unsuccessful
    pub fn try_get_mut<'a>() -> Option<tokio::sync::RwLockWriteGuard<'a, Self>> {
        ONLINE_MANAGER.try_write().ok()
    }

    
    pub async fn send_packet(&self, packet: impl Into<PacketId>) -> bool { 
        let Some(writer) = &self.writer else { return false }; 

        let data = SimpleWriter::new().write(packet.into()).done();
        match writer.lock().await.send(tokio_tungstenite::tungstenite::protocol::Message::Binary(data)).await {
            Ok(_) => true,
            Err(e) => {
                error!("Error sending data ({}:{}): {}", file!(), line!(), e);
                if let Err(e) = writer.lock().await.close().await {
                    error!("Error closing connection: {}", e);
                }
                false
            }
        }
    }
}

const LOG_PINGS:bool = false;
fn ping_handler() {
    tokio::spawn(async move {
        let duration = std::time::Duration::from_millis(1000);

        loop {
            tokio::time::sleep(duration).await;
            if LOG_PINGS { trace!("Sending ping"); };
            let s = OnlineManager::get().await;
            if s.writer.is_none() { return; }
            s.send_packet(PacketId::Ping).await;
        }
    });
}

#[allow(unused)]
pub enum SetAction {
    Idle,
    Closing,

    Listening {
        artist: String,
        title: String,

        elapsed: f32,
        duration: f32
    },

    Playing {
        artist: String,
        title: String,
        version: String,
        creator: String,
        multiplayer_lobby_name: Option<String>,
        start_time: i64,
    },

    Spectating {
        player: String,
        artist: String,
        title: String,
        creator: String,
        version: String
    }
}
impl SetAction {
    pub fn get_action(&self) -> UserAction {
        match self {
            Self::Idle => UserAction::Idle,
            Self::Closing => UserAction::Leaving,
            Self::Playing { .. } => UserAction::Ingame,
            Self::Listening { .. } => UserAction::Idle,
            Self::Spectating { .. } => UserAction::Ingame,
        }
    }
}

