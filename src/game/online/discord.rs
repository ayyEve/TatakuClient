use discord_rich_presence::{activity::{self, Assets}, DiscordIpc, DiscordIpcClient};
use crate::prelude::*;

const APP_ID:&'static str = "857981337423577109";

lazy_static::lazy_static! {
    // static ref CONTINUE_CALLBACKS:AtomicBool = AtomicBool::new(true);
}
pub struct Discord {
    client: Arc<Mutex<DiscordIpcClient>>,
    last_status: Arc<Mutex<(String, String)>>
}

impl Discord {
    pub fn new() -> TatakuResult<Self> {
        info!("Setting up Discord RPC");
        macro_rules! map_err {
            ($e:expr) => {
                $e.map_err(|e|TatakuError::String(format!("Discord Error: {e}")))?
            };
        }

        // create connection
        let mut client = map_err!(DiscordIpcClient::new(APP_ID));
        
        // connect
        info!("Connecting to Discord");
        map_err!(client.connect());

        // set initial status
        map_err!(client.set_activity(activity::Activity::new()
            .state("Tataku")
            .details("Loading...")
            .assets(Assets::new()
            .large_image("icon"))
        ));

        info!("Done");
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            last_status: Arc::new(Mutex::new((String::new(), String::new())))
        })
    }

    pub async fn change_status(&self, state:String, desc:String) {
        #[cfg(feature="discord")] {
            // check text
            {
                let mut lock = self.last_status.lock().await;
                let (c_state, c_desc) = &*lock;
                // if its the same text, exit
                if c_state == &state && c_desc == &desc {return}
                // if not, set the current text and continue
                *lock = (state.clone(), desc.clone());
            }

            info!("Setting Discord State to '{state},{desc}'");
            let mut client = self.client.lock().await;

            let mut activity = activity::Activity::new()
                .assets(Assets::new()
                    .large_image("icon-new")
                    .large_text("Tataku!")
                );
            
            if !state.is_empty() {
                activity = activity.state(&state)
            }
            if !desc.is_empty() {
                activity = activity.details(&desc)
            }

            
            if let Err(e) = client.set_activity(activity) {
                warn!("Error updating discord presence: {e}")
            } else {
                info!("Done Setting Discord State");
            }
        }
    }
}

impl Drop for Discord {
    fn drop(&mut self) {
        error!("Discord Dropping!!!!!!!!!");
    }
}