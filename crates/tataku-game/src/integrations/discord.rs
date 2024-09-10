use crate::prelude::*;
use discord_rich_presence::{
    DiscordIpc, 
    DiscordIpcClient,
    activity::{ 
        Assets, 
        Timestamps, 
        Activity
    }, 
};

// use tokio::sync::mpsc::{ Sender, Receiver, channel };

const APP_ID:&'static str = "857981337423577109";
const RECONNECT_INTERVAL: f32 = 5_000.0; // every 5 seconds try again



pub struct Discord {
    // client: Arc<Mutex<DiscordIpcClient>>,
    // last_status: Arc<AsyncMutex<(String, String)>>

    // sender: Sender<DiscordThreadMessage>,
    // receiver: Receiver<DiscordThreadMessage>,

    client: DiscordIpcClient,
    last_connection_attempt: Option<Instant>,

    enabled: bool,
    connected: bool,

    // last_status: (String, String),
}
impl Discord {
    pub fn new() -> TatakuResult<Self> {
        // let (sender, thread_receiver) = channel(5);
        // let (thread_sender, receiver) = channel(5);

        // Self::create_thread(sender, receiver);



        // // dont start discord if theres no gameplay
        // #[cfg(not(feature="gameplay"))] return Err(TatakuError::String(String::new()));

        // trace!("Setting up Discord RPC");
        // macro_rules! map_err {
        //     ($e:expr) => {
        //         $e.map_err(|e| TatakuError::String(format!("Discord Error: {e}")))?
        //     };
        // }

        // // create connection
        // let mut client = map_err!(DiscordIpcClient::new(APP_ID));
        
        // // connect
        // trace!("Connecting to Discord");
        // map_err!(client.connect());

        // // set initial status
        // map_err!(client.set_activity(Activity::new()
        //     .state("Tataku")
        //     .details("Loading...")
        //     .assets(Assets::new().large_image("icon"))
        // ));

        // trace!("Done");
        // Ok(Self {
        //     client: Arc::new(Mutex::new(client)),
        //     last_status: Arc::new(AsyncMutex::new((String::new(), String::new())))
        // })

        Ok(Self {
            // sender: thread_sender,
            // receiver: thread_receiver,

            client: DiscordIpcClient::new(APP_ID).map_err(DiscordError)?,
            connected: false,
            enabled: false,
            last_connection_attempt: None,
            // last_status: Default::default()
        })
    }

    // fn create_thread(
    //     sender: Sender<DiscordThreadMessage>, 
    //     receiver: Receiver<DiscordThreadMessage>,
    // ) {
    //     tokio::spawn(async move {
    //         let Ok(mut client) = DiscordIpcClient::new(APP_ID) else {
    //             let _ = sender.send(DiscordThreadMessage::Dropped).await;
    //             return;
    //         };
    //         client.connect().unwrap();
            
    //         loop {
    //             if let Ok(a) = client.recv() {
    //                 println!("{a:?}")
    //             }
    //         }
            


    //         while let Some(msg) = receiver.recv().await  {
    //             match msg {
    //                 DiscordThreadMessage::Dropped => return,
    //                 DiscordThreadMessage::Connect => {
    //                     if let Err(e) = client.connect() {

    //                     }
    //                 }
    //                 DiscordThreadMessage::Disconnect => {

    //                 }

    //                 DiscordThreadMessage::TatakuEvent(_event) => {

    //                 }

    //                 _ => {}
    //             }
                
    //         }

    //         let _ = sender.send(DiscordThreadMessage::Dropped).await;
    //     });
    // }

    // pub async fn change_status(&self, action_info: &SetAction, playmode: Option<String>) {
    //     let state;
    //     let mut desc = String::new();
    //     let mut timestamps = None;

    //     match &action_info {
    //         SetAction::Idle => state = format!("Idle"),
    //         SetAction::Closing => state = format!("Closing"),
            
    //         SetAction::Listening { artist, title, elapsed, duration } => {
    //             let now = chrono::Utc::now().timestamp();
    //             let start = now - (elapsed / 1000.0) as i64;
    //             let end = start + (duration / 1000.0) as i64;

    //             timestamps = Some(Timestamps::new().start(start).end(end));
    //             state = format!("{artist} - {title}");
    //         }
    //         SetAction::Spectating { player, artist, title, version, creator } => {
    //             state = format!("Watching {player}: {artist} - {title}");
    //             desc = format!("{} by {}", version, creator);
    //         }
    //         SetAction::Playing { artist, title, version, multiplayer_lobby_name:_, creator, start_time } => {
    //             timestamps = Some(Timestamps::new().start(*start_time));
    //             state = format!("{artist} - {title}");
    //             desc = format!("{} by {}", version, creator);
    //         }
    //     };


    //     // check text
    //     {
    //         let mut lock = self.last_status.lock().await;
    //         let (c_state, c_desc) = &*lock;
    //         // if its the same text, exit
    //         if c_state == &state && c_desc == &desc { return }
    //         // if not, set the current text and continue
    //         *lock = (state.clone(), desc.clone());
    //     }

    //     trace!("Setting Discord State to '{state},{desc}'");

    //     let mut activity = Activity::new();
    //     if !state.is_empty() { activity = activity.state(&state) }
    //     if !desc.is_empty() { activity = activity.details(&desc) }

    //     let mut assets = Assets::new()
    //         .large_image("icon-new")
    //         .large_text("Tataku!"); // TODO: make the username of the logged-in user

    //     if let Some(mode) = &playmode {
    //         let mode = gamemode_display_name(mode);
    //         assets = assets
    //             .small_image("icon") // TODO: use a url for the image, where if it doesnt exist, it gives some default, so we always have the mode text
    //             .small_text(mode)
    //     }
    //     activity = activity.assets(assets);
    //     if let Some(timestamps) = timestamps {
    //         activity = activity.timestamps(timestamps);
    //     }
    //     // activity = activity.buttons(vec![Button::new("User Profile", "https://google.ca")]);

        
    //     let mut client = self.client.lock();
    //     if let Err(e) = client.set_activity(activity) {
    //         warn!("Error updating discord presence: {e}")
    //     }
    // }
    

    /// attempt to reconnect
    fn reconnect(&mut self) -> TatakuResult {
        // dont connect if not gameplaying
        #[cfg(not(feature="gameplay"))] return Ok(());
        
        // dont connect if we aren't enabled, or if we're already connected
        if !self.enabled || self.connected { return Ok(()) }


        // make sure we wait at least RECONNECT_INTERVAL before trying to reconnect
        if let Some(last_check) = self.last_connection_attempt {
            if last_check.as_millis() < RECONNECT_INTERVAL { 
                return Ok(()) 
            }
        }
        self.last_connection_attempt = Some(Instant::now());

        // attempt to reconnect
        self.client.connect().map_err(DiscordError)?;

        // should be connected at this point
        self.connected = true;
        
        Ok(())
    }
}

impl TatakuIntegration for Discord {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed("Discord") }
    fn init(
        &mut self, 
        settings: &Settings,
    ) -> TatakuResult<()> {
        self.check_enabled(settings)
    }

    fn check_enabled(
        &mut self, 
        settings: &Settings,
    ) -> TatakuResult<()> {
        if self.enabled == settings.integrations.discord { return Ok(()) }
        self.enabled = settings.integrations.discord;

        // if our current state aligns with if we're enabled, dont continue
        if self.enabled == self.connected { return Ok(()) }

        if self.enabled {
            self.reconnect()?;
        } else {
            self.client.close().map_err(DiscordError)?;
            self.connected = false;
        }

        Ok(())
    }

    fn handle_event(
        &mut self, 
        event: &TatakuEvent
    ) {
        if !self.enabled || !self.connected { return }

        let mut activity = Activity::new();

        let mut assets = Assets::new()
            .large_image("icon-new")
            .large_text("Tataku!"); // TODO: make the username of the logged-in user

        match event {
            TatakuEvent::BeatmapStarted { 
                start_time,
                beatmap, 
                playmode, 
                multiplayer, 
                spectator 
            } => {
                let start_time = (*start_time) as i64;
                let artist = &beatmap.artist;
                let title = &beatmap.title;
                let creator = &beatmap.creator;
                let version = &beatmap.version;

                // TODO:!!!!!
                // assets = assets
                //     .small_image("icon") // TODO: use a url for the image, where if it doesnt exist, it gives some default, so we always have the mode text
                //     .small_text(gamemode_display_name(&**playmode));


                activity = if let Some(player) = spectator {
                    activity
                        .timestamps(Timestamps::new().start(start_time))
                        .state(format!("Watching {player}: {artist} - {title}"))
                        .details(format!("{version} by {creator}"))
                } else if let Some(_multi) = multiplayer {
                    activity
                        .timestamps(Timestamps::new().start(start_time))
                        .state(format!("Multiplaying: {artist} - {title}"))
                        .details(format!("{version} by {creator}"))
                } else {
                    activity
                        .timestamps(Timestamps::new().start(start_time))
                        .state(format!("{artist} - {title}"))
                        .details(format!("{version} by {creator}"))
                }
            }
            TatakuEvent::SongChanged { 
                artist,
                title,
                elapsed,
                duration,
                ..
            } => {
                let now = chrono::Utc::now().timestamp();
                let start = now - (elapsed / 1000.0) as i64;
                let end = start + (duration / 1000.0) as i64;
                activity = activity
                    .timestamps(Timestamps::new().start(start).end(end))
                    .state(format!("{artist} - {title}"))
                ;
            }
            TatakuEvent::BeatmapEnded => {
                activity = activity
                    .state("Idle")
                ;
            }
            // TatakuEvent::JoinedMultiplayer(_) => todo!(),
            // TatakuEvent::LeftMultiplayer => todo!(),
            // TatakuEvent::MenuEntered(_) => return,

            _ => return
        }

        if let Err(e) = self.client.set_activity(activity.assets(assets)) {
            error!("error setting discord presence: {e:?}");
        }

    }
}
impl Drop for Discord {
    fn drop(&mut self) {
        if self.connected {
            let _ = self.client.close();
        }
    }
}

struct DiscordError(Box<dyn std::error::Error>);
impl From<DiscordError> for TatakuError {
    fn from(value: DiscordError) -> Self {
        Self::String(value.0.to_string())
    }
}


// enum DiscordThreadMessage {
//     /// channel is dead
//     Dropped,


//     /// request to connect
//     Connect,

//     /// connect successful
//     Connected,

//     /// received an error from the thread
//     Error(TatakuError),


//     /// request to disconnect
//     Disconnect,

//     /// disconnect successful
//     Disconnected,


//     /// request to handle a tataku event
//     TatakuEvent(TatakuEvent),
// }

