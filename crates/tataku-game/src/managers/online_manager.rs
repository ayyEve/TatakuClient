use PacketId::*;
use crate::prelude::*;
use tokio::{ sync::Mutex, net::TcpStream };
use futures_util::{ SinkExt, StreamExt, stream::SplitSink };
use tokio_tungstenite::{ 
    MaybeTlsStream, 
    WebSocketStream, 
    tungstenite::{
        Bytes,
        protocol::Message,
    }
};

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;


// how many spectator frames do we buffer before sending?
// higher means less packet spam
const SPECTATOR_BUFFER_FLUSH_SIZE: usize = 20;

static ONLINE_MANAGER: OnceCell<Arc<AsyncRwLock<OnlineManager>>> = OnceCell::const_new();


pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub friends: HashSet<u32>, // userid is key

    /// our user's id
    pub user_id: u32,

    /// are we successfully logged in?
    pub logged_in: bool,

    /// socket writer
    pub writer: Option<WsWriter>,

    // ====== chat ======
    #[cfg(feature="graphics")]
    pub chat_messages: HashMap<ChatChannel, Vec<ChatMessage>>,

    // ====== spectator ======
    spectator_info: OnlineSpectatorInfo,

    // ====== multiplayer ======
    multiplayer_packet_queue: Vec<MultiplayerPacket>,

    event_sender: AsyncUnboundedSender<OnlineEvent>,
}

impl OnlineManager {
    pub fn build(
        event_sender: AsyncUnboundedSender<OnlineEvent>,
    ) {
        if ONLINE_MANAGER.initialized() { panic!("???") }

        let a = Self::new(event_sender);
        let _ = ONLINE_MANAGER.set(Arc::new(AsyncRwLock::new(a)));
    }

    fn new(
        event_sender: AsyncUnboundedSender<OnlineEvent>,
    ) -> Self {
        // idk why this is suddenly required but whatever
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        #[cfg(feature="graphics")] 
        let mut messages = HashMap::new();
        #[cfg(feature="graphics")]
        let channel = ChatChannel::Channel { name: "general".to_owned() };
        #[cfg(feature="graphics")]
        messages.insert(channel.clone(), vec![ChatMessage::new(
            "System".to_owned(),
            channel,
            u32::MAX,
            "this is a test message".to_owned()
        )]);

        Self {
            user_id: 0,
            logged_in: false,
            users: HashMap::new(),
            friends: HashSet::new(),
            writer: None,
            connected: false,
            #[cfg(feature="graphics")]
            chat_messages: messages,
            spectator_info: OnlineSpectatorInfo::new(0),

            multiplayer_packet_queue: Vec::new(),
            event_sender,
        }
    }

    #[cfg(feature="gameplay")]
    pub fn start(
        settings: &Settings,
        mut action_receiver: AsyncUnboundedReceiver<OnlineAction>,
    ) {
        let server_url = settings.server_url.clone();
        let username = settings.username.clone();
        let password = settings.password.clone();
        let logging_settings = settings.logging_settings;
        
        
        tokio::spawn(async move {
            info!("Starting websocket connection to url: {server_url}");
            
            // initialize the connection
            match tokio_tungstenite::connect_async(server_url).await {
                Ok((ws_stream, _)) => {
                    let (writer, mut reader) = ws_stream.split();

                    // send login
                    {
                        let mut s = Self::get_mut().await;
                        s.writer = Some(writer);
                        s.connected = true;

                        // send login packet
                        s.send_packet(Client_UserLogin {
                            protocol_version: 1,
                            game: "Tataku\n0.1.0".to_owned(),
                            username: username.clone(),
                            password: password.clone()
                        }).await;
                    }

                    
                    loop { tokio::select! {
                        message = action_receiver.recv() => {
                            let Some(message) = message else { continue };
                            match message {
                                OnlineAction::SpectateHost { host_id } => Self::start_spectating(host_id),
                                OnlineAction::StopSpectating { host_id } => Self::stop_spectating(host_id),
                                OnlineAction::SendSpectatorFrame { frame, force } => Self::send_spec_frames(vec![*frame], force),
                            }
                        }

                        message = reader.next() => {
                            let Some(message) = message else { return };

                            match message {
                                Ok(Message::Binary(data)) => {
                                    let data = data.to_vec();
                                    if let Err(e) = Self::handle_packet(data, &logging_settings).await {
                                        error!("Error with packet: {}", e);
                                    }
                                }
                                Ok(Message::Ping(_)) => {
                                    if let Some(writer) = &mut Self::get_mut().await.writer {
                                        let _ = writer.send(Message::Pong(Bytes::from_static(&[]))).await;
                                    }
                                }

                                Ok(Message::Close(_)) => {
                                    Self::send_notification(
                                        Notification::default()
                                        .text("Disconnected from server")
                                        .color(Color::RED)
                                        .duration(5000.0)
                                    ).await;

                                    Self::disconnect().await;
                                }
                                Ok(message) => if logging_settings.extra_online_logging { warn!("Got other network message: {message:?}"); },

                                Err(oof) => {
                                    error!("network connection error: {oof}\nAttempting to reconnect");
                                    Self::disconnect().await;

                                    // reconnect handled by caller
                                    break;
                                }
                            }
                        }

                    } }
                }
                Err(oof) => {
                    warn!("Could not accept connection: {oof:?}");
                }
            }
        });
    }

    /// disconnect and reset everything
    pub fn restart() {
        tokio::spawn(async {
            Self::get_mut().await.reset().await;
        });
    }

    /// disconnect and reset everything
    async fn reset(&mut self) {
        // if we currently have a connection, close it
        if let Some(writer) = &mut self.writer {
            let _ = writer.close().await;
        }

        // reset most values
        self.writer = None;
        self.user_id = 0;
        self.connected = false;
        self.logged_in = false;
        self.users.clear();
        self.friends.clear();

        self.spectator_info = OnlineSpectatorInfo::default();
        self.multiplayer_packet_queue.clear();
        self.event_sender.send(OnlineEvent::Disconnected).unwrap();
    }

    /// disconnect without resetting anything
    async fn disconnect() {
        let mut s = Self::get_mut().await;
        s.writer = None;
        s.connected = false;
        s.logged_in = false;
        s.event_sender.send(OnlineEvent::Disconnected).unwrap();
    }

    /// handle an incoming server packet
    #[cfg(feature="gameplay")]
    async fn handle_packet(data: Vec<u8>, log_settings: &LoggingSettings) -> TatakuResult<()> {
        let mut reader = SerializationReader::new(data);
        
        while reader.can_read() {
            // trace!("reading packet from server");
            let packet:PacketId = reader.read("packet id")?;
            // if log_settings.extra_online_logging { info!("Got packet {:?}", packet); };

            match packet {
                // ===== ping/pong =====
                PacketId::Ping => { Self::get_mut().await.send_packet(Pong).await; },
                PacketId::Pong => {/* trace!("Got pong from server"); */},

                // login
                PacketId::Server_LoginResponse { status, user_id } => {

                    match status {
                        LoginStatus::UnknownError => {
                            trace!("Unknown Error");
                                
                            Self::send_notification(
                                Notification::default()
                                .text("[Login] Unknown error logging in")
                                .color(Color::RED)
                                .duration(5000.0)
                            ).await;
                        }
                        LoginStatus::BadPassword => {
                            trace!("Auth failed");    
                            Self::send_notification(
                                Notification::default()
                                .text("[Login] Authentication failed")
                                .color(Color::RED)
                                .duration(5000.0)
                            ).await;
                        }
                        LoginStatus::NoUser => {
                            trace!("User not found");
                                
                            Self::send_notification(
                                Notification::default()
                                .text("[Login] Authentication failed")
                                .color(Color::RED)
                                .duration(5000.0)
                            ).await;
                        }
                        LoginStatus::NotActivated => {
                            trace!("User not activated");
                                
                            Self::send_notification(
                                Notification::default()
                                .text("[Login] Your account is pending activation")
                                .color(Color::YELLOW)
                                .duration(5000.0)
                            ).await;
                        }
                        LoginStatus::Ok => {
                            trace!("Success, got user_id: {user_id}");
                            {
                                let mut om = Self::get_mut().await;
                                om.user_id = user_id;
                                om.logged_in = true;
                                om.spectator_info = OnlineSpectatorInfo::new(user_id);
                            }
                                
                            Self::send_notification(
                                Notification::default()
                                .text("[Login] Logged in!")
                                .color(Color::GREEN)
                                .duration(2000.0)
                            ).await;

                            Self::send_event(OnlineEvent::LoggedIn { 
                                user_id, 
                                username: String::new()
                            }).await;

                            ping_handler();

                            // request friends list
                            Self::get_mut().await.send_packet(ChatPacket::Client_GetFriends).await;
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

                    Self::send_notification(Notification::default()
                        .text(message)
                        .color(color)
                        .duration(duration)
                    ).await;
                }
                // server error
                PacketId::Server_Error { code, error } => {
                    warn!("Got server error {:?}: '{}'", code, error)
                }


                // ===== user updates =====
                PacketId::Server_UserJoined { user_id, username, game } => {
                    if log_settings.extra_online_logging { debug!("User {username} joined (id: {user_id}, game: {game})"); };
                    let mut user = OnlineUser::new(user_id, username.clone());
                    user.game = game;

                    let mut s = Self::get_mut().await;
                    s.users.insert(user_id, Arc::new(Mutex::new(user)));

                    if s.friends.contains(&user_id) {
                        Self::send_notification(
                            Notification::default()
                            .text(format!("{username} is online"))
                            .duration(5000.0)
                            .color(Color::BLUE)
                        ).await;
                    }
                }
                PacketId::Server_UserLeft { user_id } => {
                    if log_settings.extra_online_logging { debug!("User id {user_id} left"); };

                    let mut lock = Self::get_mut().await;
                    // remove from online users
                    if let Some(u) = lock.users.remove(&user_id) {
                        let l = u.lock().await;
                        let username = &l.username;
                        if lock.friends.contains(&user_id) {
                            Self::send_notification(
                                Notification::default()
                                .text(format!("{username} is offline"))
                                .color(Color::BLUE)
                                .duration(5000.0)
                            ).await;
                        }
                    }

                    // remove from our spec list
                    lock.spectator_info.remove_spec(0, user_id);
                }
                PacketId::Server_UserStatusUpdate { user_id, action, action_text, mode } => {
                    // debug!("Got user status update: {}, {:?}, {} ({:?})", user_id, action, action_text, mode);
                    
                    if let Some(e) = Self::get().await.users.get(&user_id) {
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

    #[cfg(feature="gameplay")]
    async fn handle_chat_packet(packet: ChatPacket, log_settings: &LoggingSettings) -> TatakuResult<()> {
        match packet {
            ChatPacket::Server_SendMessage {sender_id, message, channel}=> {
                if log_settings.extra_online_logging { debug!("Got message: `{}` from user id `{}` in channel `{}`", message, sender_id, channel); };

                let channel = if channel.starts_with("#") {
                    ChatChannel::Channel {name: channel.trim_start_matches("#").to_owned()}
                } else {
                    ChatChannel::User {username: channel}
                };

                let mut lock = Self::get_mut().await;
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
                let mut s = Self::get_mut().await;
                for i in friend_ids.iter() {
                    if let Some(u) = s.users.get_mut(i) {
                        u.lock().await.friend = true;
                    }
                }
                info!("got friends list: {friend_ids:?}");

                s.friends = friend_ids.into_iter().collect();
            }

            ChatPacket::Server_UpdateFriend { friend_id, is_friend } => {
                let mut s = Self::get_mut().await;
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
                let mut lock = Self::get_mut().await;
                if let Some(frames) = lock.spectator_info.incoming_frames.get_mut(&host_id) {
                    frames.extend(new_frames);
                } else {
                    warn!("got spec packets for host we're not spectating: {host_id}");
                }
            }
            // spec join/leave
            SpectatorPacket::Server_SpectatorJoined { user_id, username }=> {
                Self::get_mut().await.spectator_info.add_spec(host_id, user_id, username.clone());
                Self::send_notification(
                    Notification::default()
                    .text(format!("{username} is now spectating"))
                    .color(Color::GREEN)
                    .duration(2000.0)
                ).await;
                Self::send_event(OnlineEvent::SpectatorEvent(SpectatorEvent::SpectatorJoined { user_id, username })).await;
            }
            SpectatorPacket::Server_SpectatorLeft { user_id } => {
                let user = if let Some(u) = Self::get().await.find_user_by_id(user_id) {
                    u.lock().await.username.clone()
                } else {
                    "A user".to_owned()
                };
                Self::get_mut().await.spectator_info.remove_spec(host_id, user_id);
                Self::send_notification(
                    Notification::default()
                    .text(format!("{user} stopped spectating"))
                    .color(Color::GREEN)
                    .duration(2000.0)
                ).await;
                
                Self::send_event(OnlineEvent::SpectatorEvent(SpectatorEvent::SpectatorLeft { user_id })).await;
            }
            SpectatorPacket::Server_SpectateResult { result} => {
                trace!("Got spec result {result:?}");
                let mut notif = None;
                match result {
                    SpectateResult::Ok => Self::get_mut().await.spectator_info.add_host(host_id),
                    SpectateResult::Error_SpectatingBot => notif = Some(Notification::new_text("You cannot spectate a bot!", Color::RED, 3000.0)),
                    SpectateResult::Error_HostOffline => notif = Some(Notification::new_text("Spectate host is offline!", Color::RED, 3000.0)),
                    SpectateResult::Error_SpectatingYourself => notif = Some(Notification::new_text("You cannot spectate yourself!", Color::RED, 3000.0)),
                    SpectateResult::Error_Unknown => notif = Some(Notification::new_text("Unknown error trying to spectate!", Color::RED, 3000.0)),
                }

                if let Some(notif) = notif {
                    Self::send_notification(notif).await;
                }
            }

            _ => {}
        }

        Ok(())
    }

    async fn handle_multi_packet(packet: MultiplayerPacket, _log_settings: &LoggingSettings) -> TatakuResult<()> {
        // the game handles these now

        let mut online = Self::get_mut().await;

        match packet {
            MultiplayerPacket::Server_LobbyInvite { inviter_id, lobby } => {
                let username = if let Some(user) = online.users.get(&inviter_id) {
                    user.lock().await.username.clone()
                } else {
                    "A user".to_owned()
                };

                online.event_sender.send(OnlineEvent::MultiplayerLobbyInvite { 
                    inviter_id, 
                    inviter_username: username, 
                    lobby 
                }).unwrap();
            }
            other => online.multiplayer_packet_queue.push(other),
        }
        Ok(())
    }



    /// set our user's action for the server and any enabled integrations
    pub fn set_action(action_info: SetAction, incoming_mode: Option<String>) {
        tokio::spawn(async move {
            let mut s = Self::get_mut().await;
            let mode = incoming_mode.clone().unwrap_or_default();

            let action = action_info.get_action();
            let action_text = match &action_info {
                SetAction::Idle => "Idle".to_string(),
                SetAction::Closing => "Closing".to_string(),

                SetAction::Listening { artist, title, .. } => format!("Listening to {artist} - {title}"),
                SetAction::Spectating { player, artist, title, version, creator:_ } => format!("Watching {player} play {artist} - {title}[{version}]"),
                SetAction::Playing { artist, title, version, .. } => format!("Playing {artist} - {title}[{version}]"),
            };

            s.send_packet(PacketId::Client_StatusUpdate { action, action_text: action_text.clone(), mode }).await;
            if action == UserAction::Leaving {
                s.send_packet(PacketId::Client_LogOut).await;
            }

        });
    }

    pub fn find_user_by_id(&self, user_id: u32) -> Option<Arc<Mutex<OnlineUser>>> {
        self.users.get(&user_id).cloned()
    }
}

// spectator functions
impl OnlineManager {
    pub async fn update_usernames(data: &mut CurrentLobbyInfo) {
        data.player_usernames.clear();
        let om = Self::get().await;

        for user in data.info.players.iter() {
            let Some(user) = om.users.get(&user.user_id) else { continue };
            let user = user.lock().await;
            data.player_usernames.insert(user.user_id, user.username.clone());
        }
    }

    pub fn send_spec_frames(frames: Vec<SpectatorFrame>, force_send: bool) {
        tokio::spawn(async move {
            let mut lock = Self::get_mut().await;

            lock.spectator_info.outgoing_frames.extend(frames);
            // wait at most 1s before sending packets
            let times_up = lock.spectator_info.last_sent_frame.as_millis() > 1000.0;

            if force_send || times_up || lock.spectator_info.outgoing_frames.len() >= SPECTATOR_BUFFER_FLUSH_SIZE {
                let frames = std::mem::take(&mut lock.spectator_info.outgoing_frames);
                
                // info!("Sending {} spec packets", frames.len());
                let id = lock.user_id;
                lock.send_packet(SpectatorPacket::Client_SpectatorFrames {frames}.with_host(id)).await;
                lock.spectator_info.last_sent_frame = TatakuInstant::now();
            }
        });

    }

    /// attempt to start spectating a host
    /// 
    /// this doesnt actually begin the spec process, it just sends the request to the server
    /// the process starts when the spec is approved
    pub fn start_spectating(host_id: u32) {
        tokio::spawn(async move {
            let mut s = Self::get_mut().await;
            s.send_packet(SpectatorPacket::Client_Spectate.with_host(host_id)).await;
        });
    }

    pub fn stop_spectating(host_id: u32) {
        info!("Request to stop speccing {host_id}");
        tokio::spawn(async move {
            trace!("Attempting to stop speccing {host_id}");
            let mut s = Self::get_mut().await;
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
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_AddLobbyListener).await;
        s.send_packet(MultiplayerPacket::Client_LobbyList).await;
    }
    pub async fn remove_lobby_listener() {
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_AddLobbyListener).await;
    }

    pub async fn invite_user(user_id: u32) {
        // info!("inviting user {user_id}");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyInvite { user_id } ).await;
    }

    pub async fn update_lobby_beatmap(beatmap: Arc<BeatmapMeta>, mode: String) {
        // info!("update lobby beatmap: {beatmap:?}, {mode}");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyMapChange { 
            new_map: LobbyBeatmap { 
                title: beatmap.version_string(), 
                hash: beatmap.beatmap_hash, 
                mode,
                map_game: beatmap.beatmap_type.into()
            }
        }).await;
    }

    pub async fn move_lobby_slot(new_slot: u8) {
        // info!("move slot");
        let mut s = Self::get_mut().await;
        let our_id = s.user_id;
        s.send_packet(MultiplayerPacket::Client_LobbySlotChange { slot: new_slot, new_status: LobbySlot::Filled {user: our_id} } ).await;
    }
    pub async fn update_lobby_slot(slot: u8, new_state: LobbySlot) {
        // info!("update slot");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbySlotChange { slot, new_status: new_state } ).await;
    }

    pub async fn update_lobby_state(new_state: LobbyUserState) {
        // info!("update our user state: {new_state:?}");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyUserState { new_state } ).await;
    }

    pub async fn lobby_load_complete() {
        // info!("sending load complete");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyMapLoaded).await;
    }

    pub async fn lobby_map_complete(score: Score) {
        // info!("sending map complete");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyMapComplete { score }).await;
    }

    pub async fn lobby_map_start() {
        // info!("sending map start");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyStart).await;
    }

    pub async fn lobby_update_score(score: Score) {
        // info!("update lobby score");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyScoreUpdate { score }).await;
    }

    pub async fn lobby_change_host(new_host: u32) {
        // info!("change lobby host");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyChangeHost { new_host }).await;
    }

    pub async fn lobby_update_mods(mods: HashSet<String>, speed: u16) {
        // info!("update mods and speed");
        let mut s = Self::get_mut().await;
        s.send_packet(MultiplayerPacket::Client_LobbyUserModsChanged { mods, speed }).await;
    }
}

impl OnlineManager {
    /// opens a read lock on the online manager
    async fn get<'a>() -> tokio::sync::RwLockReadGuard<'a, Self> {
        ONLINE_MANAGER.get().unwrap().read().await
    }
    /// opens a write lock on the online manager
    async fn get_mut<'a>() -> tokio::sync::RwLockWriteGuard<'a, Self> {
        ONLINE_MANAGER.get().unwrap().write().await
    }

    pub async fn get_user(id: u32) -> Option<OnlineUser> {
        Some(Self::get().await.users.get(&id)?.lock().await.clone())
    }

    async fn send_event(event: OnlineEvent) {
        Self::get().await.event_sender.send(event).unwrap();
    }
    async fn send_notification(notif: Notification) {
        Self::send_event(OnlineEvent::TatakuAction(notif.into())).await
    }

    
    pub async fn send_packet(&mut self, packet: impl Into<PacketId>) -> bool { 
        let Some(writer) = &mut self.writer else { return false }; 
        let packet = packet.into();

        let data = SimpleWriter::new().write(packet).done();
        match writer.send(Message::Binary(Bytes::from_owner(data))).await {
            Ok(_) => true,
            Err(e) => {
                error!("Error sending data ({}:{}): {}", file!(), line!(), e);
                if let Err(e) = writer.close().await {
                    error!("Error closing connection: {}", e);
                }
                false
            }
        }
    }


    pub fn send_packet_static(packet: impl Into<PacketId> + Send + Sync + 'static) {
        tokio::spawn(async move {
            let mut om = Self::get_mut().await;
            om.send_packet(packet).await;
        });
    }
}

const LOG_PINGS:bool = false;
fn ping_handler() {
    #[cfg(feature="gameplay")]
    tokio::spawn(async move {
        let duration = std::time::Duration::from_millis(1000);

        loop {
            tokio::time::sleep(duration).await;
            if LOG_PINGS { trace!("Sending ping"); };
            let mut s = OnlineManager::get_mut().await;
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
