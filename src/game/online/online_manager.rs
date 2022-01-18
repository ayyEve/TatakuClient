use tokio::{sync::Mutex, net::TcpStream};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message};

use crate::prelude::*;
use super::discord::Discord;
use super::online_user::OnlineUser;

use PacketId::*;

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;


const EXTRA_ONLINE_LOGGING:bool = true;

// how many frames do we buffer before sending?
// higher means less packet spam
const SPECTATOR_BUFFER_FLUSH_SIZE: usize = 20;
type ThreadSafeSelf = Arc<Mutex<OnlineManager>>;

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
                    println!("[Writer] Error sending data ({}:{}): {}", file!(), line!(), e);
                    if let Err(e) = writer.lock().await.close().await {
                        println!("[Writer] error closing connection: {}", e);
                    }
                    false
                }
            }
        } else {
            false
        }
    }
}



lazy_static::lazy_static! {
    ///TODO: somehow change this to a RwLock. it should prioritize reads, as reads will almost always be syncronous
    pub static ref ONLINE_MANAGER:ThreadSafeSelf = Arc::new(Mutex::new(OnlineManager::new()));
}

///
pub struct OnlineManager {
    pub connected: bool,
    pub users: HashMap<u32, Arc<Mutex<OnlineUser>>>, // user id is key
    pub discord: Discord,

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
    pub(crate) buffered_spectator_frames: SpectatorFrames,
    pub(crate) last_spectator_frame: Instant,

    pub(crate) spectator_list: Vec<(u32, String)>,
    /// which users are waiting for a spectator info response?
    /// TODO: should probably move the list itself to the server
    pub(crate) spectate_info_pending: Vec<u32>,

    /// was a spectator request accepted? if so, this will be the user_id
    spectate_pending: u32
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
            discord: Discord::new(),
            // chat: Chat::new(),
            writer: None,
            connected: false,
            buffered_spectator_frames: Vec::new(),
            last_spectator_frame: Instant::now(),
            spectating: false,

            spectator_list: Vec::new(),
            spectate_info_pending: Vec::new(),
            chat_messages: messages,
            spectate_pending: 0
        }
    }
    pub async fn start() {
        let s = ONLINE_MANAGER.clone();
        // initialize the connection
        match connect_async(Settings::get().server_url).await {
            Ok((ws_stream, _)) => {
                s.lock().await.connected = true;
                let (writer, mut reader) = ws_stream.split();
                let writer = Arc::new(Mutex::new(writer));

                // send login
                {
                    let mut s = s.lock().await;
                    s.writer = Some(writer);
                    let settings = Settings::get();

                    let password = sha512(settings.password);

                    // send login packet
                    send_packet!(s.writer, create_packet!(Client_UserLogin {
                        protocol_version: 1,
                        game: "Tataku\n0.1.0".to_owned(),
                        username: settings.username.clone(),
                        password
                    }));
                }

                while let Some(message) = reader.next().await {
                    match message {
                        Ok(Message::Binary(data)) => OnlineManager::handle_packet(data).await,
                        Ok(Message::Ping(_)) => {
                            if let Some(writer) = s.lock().await.writer.as_mut() {
                                let _ = writer.lock().await.send(Message::Pong(Vec::new())).await;
                            }
                        }
                        Ok(message) => if EXTRA_ONLINE_LOGGING {println!("[Online] got something else: {:?}", message)},

                        Err(oof) => {
                            println!("[Online] oof: {}", oof);
                            s.lock().await.connected = false;
                            s.lock().await.writer = None;
                            // reconnect?
                            break;
                        }
                    }
                }
            }
            Err(oof) => {
                s.lock().await.connected = false;
                println!("[Online] could not accept connection: {}", oof);
            }
        }
    }

    async fn handle_packet(data:Vec<u8>) {
        let s = ONLINE_MANAGER.clone();
        let mut reader = SerializationReader::new(data);

        while reader.can_read() {
            let packet:PacketId = reader.read();
            if EXTRA_ONLINE_LOGGING {println!("[Online] got packet {:?}", packet)};

            match packet {
                // ===== ping/pong =====
                PacketId::Ping => {send_packet!(s.lock().await.writer, create_packet!(Pong));},
                PacketId::Pong => {/* println!("[Online] got pong from server"); */},

                // login
                PacketId::Server_LoginResponse { status, user_id } => {
                    match status {
                        LoginStatus::UnknownError => {
                            println!("[Login] Unknown Error");
                            NotificationManager::add_text_notification("[Login] Unknown error logging in", 5000.0, Color::RED);
                        },
                        LoginStatus::BadPassword => {
                            println!("[Login] auth failed");
                            NotificationManager::add_text_notification("[Login] Authentication failed", 5000.0, Color::RED);
                        },
                        LoginStatus::NoUser => {
                            println!("[Login] user not found");
                            NotificationManager::add_text_notification("[Login] Authentication failed", 5000.0, Color::RED);
                        },
                        LoginStatus::Ok => {
                            println!("[Login] success, got user_id: {}", user_id);
                            s.lock().await.user_id = user_id;
                            NotificationManager::add_text_notification("[Login] Logged in!", 2000.0, Color::GREEN);

                            ping_handler()
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

                    NotificationManager::add_text_notification(&message, duration, color);
                }
                // server error
                PacketId::Server_Error { code, error } => {
                    println!("[Online] got server error {:?}: '{}'", code, error)
                }


                // ===== user updates =====
                PacketId::Server_UserJoined { user_id, username, game } => {
                    if EXTRA_ONLINE_LOGGING {println!("[Online] user {} joined (id: {})", username, user_id)};
                    let mut user = OnlineUser::new(user_id, username);
                    user.game = game;
                    s.lock().await.users.insert(user_id, Arc::new(Mutex::new(user)));
                }
                PacketId::Server_UserLeft {user_id} => {
                    if EXTRA_ONLINE_LOGGING {println!("[Online] user id {} left", user_id)};

                    let mut lock = s.lock().await;
                    // remove from online users
                    lock.users.remove(&user_id);

                    // remove from our spec list
                    for (i, &(id, _)) in lock.spectator_list.iter().enumerate() {
                        if id == user_id {
                            lock.spectator_list.swap_remove(i);
                            break;
                        }
                    }
                }
                PacketId::Server_UserStatusUpdate { user_id, action, action_text, mode } => {
                    // println!("[Online] got user status update: {}, {:?}, {} ({:?})", user_id, action, action_text, mode);
                    
                    if let Some(e) = s.lock().await.users.get_mut(&user_id) {
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
                    if EXTRA_ONLINE_LOGGING {println!("[Online] got message: `{}` from user id `{}` in channel `{}`", message, sender_id, channel)};

                    let channel = if channel.starts_with("#") {
                        ChatChannel::Channel {name: channel.trim_start_matches("#").to_owned()}
                    } else {
                        ChatChannel::User {username: channel}
                    };

                    let mut lock = s.lock().await;
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

                
                // ===== spectator =====
                PacketId::Server_SpectatorFrames { frames } => {
                    // println!("[Online] got {} spectator frames from the server", frames.len());
                    let mut lock = s.lock().await;
                    lock.buffered_spectator_frames.extend(frames);
                }
                // spec join/leave
                PacketId::Server_SpectatorJoined { user_id, username }=> {
                    s.lock().await.spectator_list.push((user_id, username.clone()));
                    NotificationManager::add_text_notification(&format!("{} is now spectating", username), 2000.0, Color::GREEN);
                }
                PacketId::Server_SpectatorLeft { user_id } => {
                    let user = if let Some(u) = s.lock().await.find_user_by_id(user_id) {
                        u.lock().await.username.clone()
                    } else {
                        "A user".to_owned()
                    };
                    s.lock().await.spectator_list.remove_item((user_id, user.clone()));
                    
                    NotificationManager::add_text_notification(&format!("{} stopped spectating", user), 2000.0, Color::GREEN);
                }
                PacketId::Server_SpectateResult {result, host_id} => {
                    println!("[Online] got spec result {:?}", result);
                    match result {
                        SpectateResult::Ok => s.lock().await.spectate_pending = host_id,
                        SpectateResult::Error_SpectatingBot => NotificationManager::add_text_notification("You cannot spectate a bot!", 3000.0, Color::RED),
                        SpectateResult::Error_HostOffline => NotificationManager::add_text_notification("Spectate host is offline!", 3000.0, Color::RED),
                        SpectateResult::Error_SpectatingYourself => NotificationManager::add_text_notification("You cannot spectate yourself!", 3000.0, Color::RED),
                        SpectateResult::Error_Unknown => NotificationManager::add_text_notification("Unknown error trying to spectate!", 3000.0, Color::RED),
                    }
                }

                // spec info request
                PacketId::Server_SpectatorPlayingRequest {user_id} => {
                    s.lock().await.spectate_info_pending.push(user_id);
                    println!("[Online] got playing request");
                }

                // other packets
                PacketId::Unknown => {
                    println!("[Online] got unknown packet, dropping remaining packets");
                    break;
                }

                p => {
                    println!("[Online] Got unhandled packet: {:?}, dropping remaining packets", p);
                    break;
                }
            }
        }
    }

    pub fn set_action(action:UserAction, action_text:String, mode: PlayMode) {
        let c = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let mut s = c.lock().await;
            send_packet!(s.writer, create_packet!(Client_StatusUpdate {action, action_text: action_text.clone(), mode}));
            if action == UserAction::Leaving {
                send_packet!(s.writer, create_packet!(Client_LogOut));
            }
            s.discord.change_status(action_text.clone());
        });
    }


    // do things which require a reference to game
    pub fn do_game_things(&mut self, game: &mut Game) { 
        if self.spectate_pending > 0 {
            println!("[Online] speccing {}", self.spectate_pending);
            self.buffered_spectator_frames.clear();
            self.spectating = true;
            self.spectate_pending = 0;
            game.queue_state_change(GameState::Spectating(SpectatorManager::new()));
        }
        
        if self.spectate_info_pending.len() > 0 {

            // only get info if the current mode is ingame
            match &mut game.current_state {
                GameState::Ingame(manager) => {
                    for user_id in self.spectate_info_pending.iter() {
                        println!("[Online] sending playing request");
                        let packet = SpectatorFrameData::PlayingResponse {
                            user_id: *user_id,
                            beatmap_hash: manager.beatmap.hash(),
                            mode: manager.gamemode.playmode(),
                            mods: serde_json::to_string(&(*manager.current_mods)).unwrap(),
                            current_time: manager.time()
                        };

                        let clone = self.writer.clone();
                        tokio::spawn(async move {
                            let frames = vec![(0.0, packet)];
                            send_packet!(clone, create_packet!(Client_SpectatorFrames {frames}));
                            println!("[Online] playing request sent");
                        });
                    }
                    
                    self.spectate_info_pending.clear();
                }


                GameState::InMenu(menu) => {
                    match &*menu.lock().get_name() {
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
        for (&id, user) in self.users.iter() {
            if id == user_id {
                return Some(user.clone())
            }
        }

        None
    }
}
impl OnlineManager {
    pub fn send_spec_frames(frames:SpectatorFrames, force_send: bool) {
        let s = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let mut lock = s.lock().await;
            // if we arent speccing, exit
            // hopefully resolves a bug
            // if !lock.spectating {return}

            lock.buffered_spectator_frames.extend(frames);
            let times_up = lock.last_spectator_frame.elapsed().as_secs_f32() > 1.0;

            if force_send || times_up || lock.buffered_spectator_frames.len() >= SPECTATOR_BUFFER_FLUSH_SIZE {
                let frames = std::mem::take(&mut lock.buffered_spectator_frames);
                // if force_send {println!("[Online] sending spec buffer (force)")} else if times_up {println!("[Online] sending spec buffer (time)")} else {println!("[Online] sending spec buffer (len)")}

                // for i in frames.iter() {
                //     println!("writing spec packet")
                // }
                
                println!("[Online] sending {} spec packets", frames.len());
                send_packet!(lock.writer, create_packet!(Client_SpectatorFrames {frames}));
                lock.last_spectator_frame = Instant::now();
            }
        });

    }

    pub fn start_spectating(host_id: u32) {
        let s = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let mut s = s.lock().await;
            send_packet!(s.writer, create_packet!(Client_Spectate {host_id:host_id}));
        });
    }

    pub fn stop_spectating() {
        let s = ONLINE_MANAGER.clone();
        tokio::spawn(async move {
            let mut s = s.lock().await;
            s.buffered_spectator_frames.clear();
            if !s.spectating {return}
            s.spectating = false;
            println!("[Online] stop speccing");
            
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
            if LOG_PINGS {println!("[Ping] sending ping")};
            send_packet!(ONLINE_MANAGER.lock().await.writer, ping.clone());
        }
    });
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
                println!("online thread {} stopped", i);
            });
        }
    }
}
