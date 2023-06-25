use PacketId::*;
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
    pub static ref ONLINE_MANAGER:ThreadSafeSelf = Arc::new(AsyncRwLock::new(OnlineManager::new()));
}

#[macro_export]
macro_rules! create_packet {
    ($($item:expr),+) => {
        SimpleWriter::new()
        $(.write($item))+
        .done()
    };
}

#[macro_export]
macro_rules! send_packet {
    ($writer:expr, $data:expr) => {
        if let Some(writer) = &$writer {
            match writer.lock().await.send(tokio_tungstenite::tungstenite::protocol::Message::Binary($data)).await {
                Ok(_) => true,
                Err(e) => {
                    error!("Error sending data ({}:{}): {}", file!(), line!(), e);
                    if let Err(e) = writer.lock().await.close().await {
                        error!("Error closing connection: {}", e);
                    }
                    false
                }
            }
        } else {
            false
        }
    }
}


///
pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub friends: HashSet<u32>, // userid is key
    pub discord: Option<Discord>,

    pub user_id: u32, // this user's id

    /// socket writer
    pub writer: Option<Arc<Mutex<WsWriter>>>,

    // ====== chat ======
    pub chat_messages: HashMap<ChatChannel, Vec<ChatMessage>>,

    // ====== spectator ======

    // buffers 
    // is this user spectating someone?
    pub spectating: bool,
    /// buffer for incoming and outgoing spectator frames
    pub buffered_spectator_frames: SpectatorFrames,
    pub last_spectator_frame: Instant,

    pub spectator_list: Vec<(u32, String)>,
    /// which users are waiting for a spectator info response?
    /// TODO: should probably move the list itself to the server
    pub spectate_info_pending: Vec<u32>,

    /// was a spectator request accepted? if so, this will be the user_id
    spectate_pending: u32,
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
            users: HashMap::new(),
            friends: HashSet::new(),
            discord: None,
            // chat: Chat::new(),
            writer: None,
            connected: false,
            buffered_spectator_frames: Vec::new(),
            last_spectator_frame: Instant::now(),
            spectating: false,

            spectator_list: Vec::new(),
            spectate_info_pending: Vec::new(),
            chat_messages: messages,
            spectate_pending: 0,
        }
    }

    pub async fn init_discord() {
        match Discord::new() {
            Ok(discord) => ONLINE_MANAGER.write().await.discord = Some(discord),
            Err(e) => error!("discord error: {e}"),
        }
    }

    pub async fn start() {
        info!("starting network connection");
        let mut settings = SettingsHelper::new();

        // initialize the connection
        match tokio_tungstenite::connect_async(settings.server_url.clone()).await {
            Ok((ws_stream, _)) => {
                let (writer, mut reader) = ws_stream.split();
                let writer = Arc::new(Mutex::new(writer));

                // send login
                {
                    let mut s = ONLINE_MANAGER.write().await;
                    s.writer = Some(writer);
                    s.connected = true;

                    // send login packet
                    send_packet!(s.writer, create_packet!(Client_UserLogin {
                        protocol_version: 1,
                        game: "Tataku\n0.1.0".to_owned(),
                        username: settings.username.clone(),
                        password: settings.password.clone()
                    }));
                }

                while let Some(message) = reader.next().await {
                    settings.update();

                    match message {
                        Ok(Message::Binary(data)) => {
                            if let Err(e) = OnlineManager::handle_packet(data, &settings.logging_settings).await {
                                error!("Error with packet: {}", e);
                            }
                        }
                        Ok(Message::Ping(_)) => {
                            if let Some(writer) = &ONLINE_MANAGER.read().await.writer {
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
            ONLINE_MANAGER.write().await.reset().await;

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
        self.connected = false;
        self.user_id = 0;
        self.users.clear();
        self.friends.clear();

        self.spectating = false;
        self.buffered_spectator_frames.clear();
        self.spectator_list.clear();

        // TODO: move these
        self.spectate_info_pending.clear();
        self.spectate_pending = 0;
    }

    /// disconnect without resetting anything
    async fn disconnect() {
        let mut s = ONLINE_MANAGER.write().await;
        s.connected = false;
        s.writer = None;
    }

    /// handle an incoming server packet
    async fn handle_packet(data:Vec<u8>, log_settings: &LoggingSettings) -> TatakuResult<()> {
        let mut reader = SerializationReader::new(data);

        while reader.can_read() {
            let packet:PacketId = reader.read()?;
            if log_settings.extra_online_logging { debug!("Got packet {:?}", packet); };

            match packet {
                // ===== ping/pong =====
                PacketId::Ping => {send_packet!(ONLINE_MANAGER.read().await.writer, create_packet!(Pong));},
                PacketId::Pong => {/* trace!("Got pong from server"); */},

                // login
                PacketId::Server_LoginResponse { status, user_id } => {
                    match status {
                        LoginStatus::UnknownError => {
                            trace!("Unknown Error");
                            NotificationManager::add_text_notification("[Login] Unknown error logging in", 5000.0, Color::RED).await;
                        },
                        LoginStatus::BadPassword => {
                            trace!("Auth failed");
                            NotificationManager::add_text_notification("[Login] Authentication failed", 5000.0, Color::RED).await;
                        },
                        LoginStatus::NoUser => {
                            trace!("User not found");
                            NotificationManager::add_text_notification("[Login] Authentication failed", 5000.0, Color::RED).await;
                        },
                        LoginStatus::Ok => {
                            trace!("Success, got user_id: {}", user_id);
                            ONLINE_MANAGER.write().await.user_id = user_id;
                            NotificationManager::add_text_notification("[Login] Logged in!", 2000.0, Color::GREEN).await;

                            ping_handler();

                            // request friends list
                            send_packet!(ONLINE_MANAGER.read().await.writer, create_packet!(PacketId::Client_GetFriends));
                        },
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

                    let mut s = ONLINE_MANAGER.write().await;
                    s.users.insert(user_id, Arc::new(Mutex::new(user)));

                    // if this is us, make sure we have the correct username text
                    if s.user_id == user_id {
                        let mut settings = get_settings_mut!();
                        if settings.username != username {
                            settings.username = username.clone()
                        }
                    }

                    if s.friends.contains(&user_id) {
                        NotificationManager::add_text_notification(format!("{username} is online"), 5000.0, Color::BLUE).await;
                    }
                }
                PacketId::Server_UserLeft {user_id} => {
                    if log_settings.extra_online_logging { debug!("User id {} left", user_id); };

                    let mut lock = ONLINE_MANAGER.write().await;
                    // remove from online users
                    if let Some(u) = lock.users.remove(&user_id) {
                        let l = u.lock().await;
                        let username = &l.username;
                        if lock.friends.contains(&user_id) {
                            NotificationManager::add_text_notification(format!("{username} is offline"), 5000.0, Color::BLUE).await;
                        }
                    }

                    // remove from our spec list
                    for (i, &(id, _)) in lock.spectator_list.iter().enumerate() {
                        if id == user_id {
                            lock.spectator_list.swap_remove(i);
                            break;
                        }
                    }
                    
                }
                PacketId::Server_UserStatusUpdate { user_id, action, action_text, mode } => {
                    // debug!("Got user status update: {}, {:?}, {} ({:?})", user_id, action, action_text, mode);
                    
                    if let Some(e) = ONLINE_MANAGER.read().await.users.get(&user_id) {
                        let mut a = e.lock().await;
                        a.action = Some(action);
                        a.action_text = Some(action_text);
                        a.mode = Some(mode);
                    }
                }

                // score 
                PacketId::Server_ScoreUpdate { .. } => {}

                // ===== chat =====
                PacketId::Server_SendMessage {sender_id, message, channel}=> {
                    if log_settings.extra_online_logging { debug!("Got message: `{}` from user id `{}` in channel `{}`", message, sender_id, channel); };

                    let channel = if channel.starts_with("#") {
                        ChatChannel::Channel {name: channel.trim_start_matches("#").to_owned()}
                    } else {
                        ChatChannel::User {username: channel}
                    };

                    let mut lock = ONLINE_MANAGER.write().await;
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
                PacketId::Server_FriendsList { friend_ids } => {
                    let mut s = ONLINE_MANAGER.write().await;
                    for i in friend_ids.iter() {
                        if let Some(u) = s.users.get_mut(i) {
                            u.lock().await.friend = true;
                        }
                    }
                    info!("got friends list: {friend_ids:?}");

                    s.friends = friend_ids.into_iter().collect();
                }


                PacketId::Server_UpdateFriend { friend_id, is_friend } => {
                    let mut s = ONLINE_MANAGER.write().await;
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

                
                // ===== spectator =====
                PacketId::Server_SpectatorFrames { frames } => {
                    // debug!("Got {} spectator frames from the server", frames.len());
                    let mut lock = ONLINE_MANAGER.write().await;
                    lock.buffered_spectator_frames.extend(frames);
                }
                // spec join/leave
                PacketId::Server_SpectatorJoined { user_id, username }=> {
                    ONLINE_MANAGER.write().await.spectator_list.push((user_id, username.clone()));
                    NotificationManager::add_text_notification(&format!("{} is now spectating", username), 2000.0, Color::GREEN).await;
                }
                PacketId::Server_SpectatorLeft { user_id } => {
                    let user = if let Some(u) = ONLINE_MANAGER.read().await.find_user_by_id(user_id) {
                        u.lock().await.username.clone()
                    } else {
                        "A user".to_owned()
                    };
                    ONLINE_MANAGER.write().await.spectator_list.remove_item((user_id, user.clone()));
                    
                    NotificationManager::add_text_notification(&format!("{} stopped spectating", user), 2000.0, Color::GREEN).await;
                }
                PacketId::Server_SpectateResult {result, host_id} => {
                    trace!("Got spec result {:?}", result);
                    match result {
                        SpectateResult::Ok => ONLINE_MANAGER.write().await.spectate_pending = host_id,
                        SpectateResult::Error_SpectatingBot => NotificationManager::add_text_notification("You cannot spectate a bot!", 3000.0, Color::RED).await,
                        SpectateResult::Error_HostOffline => NotificationManager::add_text_notification("Spectate host is offline!", 3000.0, Color::RED).await,
                        SpectateResult::Error_SpectatingYourself => NotificationManager::add_text_notification("You cannot spectate yourself!", 3000.0, Color::RED).await,
                        SpectateResult::Error_Unknown => NotificationManager::add_text_notification("Unknown error trying to spectate!", 3000.0, Color::RED).await,
                    }
                }

                // spec info request
                PacketId::Server_SpectatorPlayingRequest {user_id} => {
                    ONLINE_MANAGER.write().await.spectate_info_pending.push(user_id);
                    trace!("Got playing request");
                }

                


                // other packets
                PacketId::Unknown => {
                    warn!("Got unknown packet, dropping remaining packets");
                    break;
                }

                p => {
                    warn!("Got unhandled packet: {:?}, dropping remaining packets", p);

                    break;
                }
            }
        }

        Ok(())
    }


    /// set our user's action for the server and any enabled integrations
    pub fn set_action(action_info: SetAction, incoming_mode: Option<PlayMode>) {
        tokio::spawn(async move {
            let s = ONLINE_MANAGER.read().await;
            let mode = incoming_mode.clone().unwrap_or(String::new());


            let action = action_info.get_action();
            let action_text = match &action_info {
                SetAction::Idle => format!("Idle"),
                SetAction::Closing => format!("Closing"),

                SetAction::Listening { artist, title, .. } => format!("Listening to {artist} - {title}"),
                SetAction::Spectating { player, artist, title, version, creator:_ } => format!("Watching {player} play {artist} - {title}[{version}]"),
                SetAction::Playing { artist, title, version, .. } => format!("Playing {artist} - {title}[{version}]"),
            };


            send_packet!(s.writer, create_packet!(Client_StatusUpdate { action, action_text: action_text.clone(), mode }));
            if action == UserAction::Leaving {
                send_packet!(s.writer, create_packet!(Client_LogOut));
            }


            if let Some(discord) = &s.discord {
                discord.change_status(&action_info, incoming_mode).await;
            }

            if get_settings!().integrations.lastfm {
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


    // do things which require a reference to game
    pub async fn do_game_things(&mut self, game: &mut Game) { 
        if self.spectate_pending > 0 {
            trace!("Speccing {}", self.spectate_pending);
            self.buffered_spectator_frames.clear();
            self.spectating = true;
            self.spectate_pending = 0;
            game.queue_state_change(GameState::Spectating(SpectatorManager::new().await));
        }
        
        if self.spectate_info_pending.len() > 0 {

            // only get info if the current mode is ingame
            match &mut game.current_state {
                GameState::Ingame(manager) => {
                    for user_id in self.spectate_info_pending.iter() {
                        trace!("Sending playing request");
                        let packet = SpectatorFrameData::PlayingResponse {
                            user_id: *user_id,
                            beatmap_hash: manager.beatmap.hash(),
                            mode: manager.gamemode.playmode(),
                            mods: manager.score.mods_string_sorted(),
                            current_time: manager.time(),
                            speed: manager.current_mods.speed
                        };

                        let clone = self.writer.clone();
                        tokio::spawn(async move {
                            let frames = vec![(0.0, packet)];
                            send_packet!(clone, create_packet!(Client_SpectatorFrames {frames}));
                            trace!("Playing request sent");
                        });
                    }
                    
                    self.spectate_info_pending.clear();
                }


                GameState::InMenu(menu) => {
                    match &*menu.get_name() {
                        // if in a pause menu, dont clear the list, the user could enter the game again
                        // so we want to wait until they decide if they want to play or quit
                        "pause" => {}
                        _ => self.spectate_info_pending.clear()
                    }
                }

                // clear list for any other mode
                GameState::Closing
                | GameState::None
                | GameState::Spectating(_) => {
                    self.spectate_info_pending.clear();
                }
            }

        }
    }

    pub fn find_user_by_id(&self, user_id: u32) -> Option<Arc<Mutex<OnlineUser>>> {
        self.users.get(&user_id).cloned()
    }
}

// spectator functions
impl OnlineManager {
    pub fn send_spec_frames(frames:SpectatorFrames, force_send: bool) {
        let s = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let mut lock = s.write().await;
            // if we arent speccing, exit
            // hopefully resolves a bug
            // if !lock.spectating {return}

            lock.buffered_spectator_frames.extend(frames);
            let times_up = lock.last_spectator_frame.elapsed().as_secs_f32() > 1.0;

            if force_send || times_up || lock.buffered_spectator_frames.len() >= SPECTATOR_BUFFER_FLUSH_SIZE {
                let frames = std::mem::take(&mut lock.buffered_spectator_frames);
                // if force_send {trace!("Sending spec buffer (force)")} else if times_up {trace!("Sending spec buffer (time)")} else {trace!("Sending spec buffer (len)")}

                // for i in frames.iter() {
                //     trace!("writing spec packet")
                // }
                
                // debug!("Sending {} spec packets", frames.len());
                send_packet!(lock.writer, create_packet!(Client_SpectatorFrames {frames}));
                lock.last_spectator_frame = Instant::now();
            }
        });

    }

    pub fn start_spectating(host_id: u32) {
        let s = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let s = s.read().await;
            send_packet!(s.writer, create_packet!(Client_Spectate {host_id:host_id}));
        });
    }

    pub fn stop_spectating() {
        let s = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let mut s = s.write().await;
            s.buffered_spectator_frames.clear();
            if !s.spectating {return}
            s.spectating = false;
            trace!("Stop speccing");
            
            send_packet!(s.writer, create_packet!(Client_LeaveSpectator));
        });
    }

    pub fn get_pending_spec_frames(&mut self) -> SpectatorFrames {
        std::mem::take(&mut self.buffered_spectator_frames)
    }
}



const LOG_PINGS:bool = false;
fn ping_handler() {
    tokio::spawn(async move {
        let ping = create_packet!(Ping);
        let duration = std::time::Duration::from_millis(1000);

        loop {
            tokio::time::sleep(duration).await;
            if LOG_PINGS { trace!("Sending ping"); };
            let writer = &ONLINE_MANAGER.read().await.writer;
            if writer.is_none() { return; }
            send_packet!(writer, ping.clone());
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

// tests
#[allow(unused_imports, dead_code)]
mod tests {
    const CONNECTION_COUNT: usize = 50;
    use crate::prelude::*;
    
    #[test]
    fn test() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            load_test().await
        });
        loop {}
    }

    async fn load_test() {
        for i in 0..CONNECTION_COUNT {
            tokio::spawn(async move {
                // let thing = super::OnlineManager::new();
                // let thing = Arc::new(tokio::sync::Mutex::new(thing));

                super::OnlineManager::start().await;
                trace!("online thread {} stopped", i);
            });
        }
    }
}
