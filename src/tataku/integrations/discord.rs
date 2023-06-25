use discord_rich_presence::{activity::{Assets, Timestamps, Activity}, DiscordIpc, DiscordIpcClient};
use crate::prelude::*;

const APP_ID:&'static str = "857981337423577109";

pub struct Discord {
    client: Arc<Mutex<DiscordIpcClient>>,
    last_status: Arc<AsyncMutex<(String, String)>>
}

impl Discord {
    pub fn new() -> TatakuResult<Self> {
        trace!("Setting up Discord RPC");
        macro_rules! map_err {
            ($e:expr) => {
                $e.map_err(|e|TatakuError::String(format!("Discord Error: {e}")))?
            };
        }

        // create connection
        let mut client = map_err!(DiscordIpcClient::new(APP_ID));
        
        // connect
        trace!("Connecting to Discord");
        map_err!(client.connect());

        // set initial status
        map_err!(client.set_activity(Activity::new()
            .state("Tataku")
            .details("Loading...")
            .assets(Assets::new().large_image("icon"))
        ));

        trace!("Done");
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            last_status: Arc::new(AsyncMutex::new((String::new(), String::new())))
        })
    }

    pub async fn change_status(&self, action_info: &SetAction, playmode: Option<PlayMode>) {
        let state;
        let mut desc = String::new();
        let mut timestamps = None;

        match &action_info {
            SetAction::Idle => state = format!("Idle"),
            SetAction::Closing => state = format!("Closing"),
            
            SetAction::Listening { artist, title, elapsed, duration } => {
                let now = chrono::Utc::now().timestamp();
                let start = now - (elapsed / 1000.0) as i64;
                let end = start + (duration / 1000.0) as i64;

                timestamps = Some(Timestamps::new().start(start).end(end));
                state = format!("{artist} - {title}");
            }
            SetAction::Spectating { player, artist, title, version, creator } => {
                state = format!("Watching {player}: {artist} - {title}");
                desc = format!("{} by {}", version, creator);
            }
            SetAction::Playing { artist, title, version, multiplayer_lobby_name:_, creator, start_time } => {
                timestamps = Some(Timestamps::new().start(*start_time));
                state = format!("{artist} - {title}");
                desc = format!("{} by {}", version, creator);
            }
        };


        // check text
        {
            let mut lock = self.last_status.lock().await;
            let (c_state, c_desc) = &*lock;
            // if its the same text, exit
            if c_state == &state && c_desc == &desc { return }
            // if not, set the current text and continue
            *lock = (state.clone(), desc.clone());
        }

        trace!("Setting Discord State to '{state},{desc}'");

        let mut activity = Activity::new();
        if !state.is_empty() { activity = activity.state(&state) }
        if !desc.is_empty() { activity = activity.details(&desc) }

        let mut assets = Assets::new()
            .large_image("icon-new")
            .large_text("Tataku!"); // TODO: make the username of the logged-in user

        if let Some(mode) = &playmode {
            let mode = gamemode_display_name(mode);
            assets = assets
                .small_image("icon") // TODO: use a url for the image, where if it doesnt exist, it gives some default, so we always have the mode text
                .small_text(mode)
        }
        activity = activity.assets(assets);
        if let Some(timestamps) = timestamps {
            activity = activity.timestamps(timestamps);
        }
        // activity = activity.buttons(vec![Button::new("User Profile", "https://google.ca")]);

        
        let mut client = self.client.lock();
        if let Err(e) = client.set_activity(activity) {
            warn!("Error updating discord presence: {e}")
        }
    }
    
}

impl Drop for Discord {
    fn drop(&mut self) {
        let _ = self.client.lock().close();
    }
}