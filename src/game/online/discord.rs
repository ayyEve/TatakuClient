use discord_rich_presence::{activity::{self, Assets}, DiscordIpc, DiscordIpcClient};
use crate::prelude::*;

const APP_ID:&'static str = "857981337423577109";

lazy_static::lazy_static! {
    // static ref CONTINUE_CALLBACKS:AtomicBool = AtomicBool::new(true);
}
pub struct Discord {
    client: Arc<Mutex<DiscordIpcClient>>
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
            client: Arc::new(Mutex::new(client))
        })
    }

    pub async fn change_status(&self, desc:String) {
        #[cfg(feature="discord")] {
            info!("Setting Discord State to '{desc}'");
            let mut client = self.client.lock().await;

            
            if let Err(e) = client.set_activity(activity::Activity::new()
                .state("Tataku")
                .details(&desc)
                .assets(Assets::new()
                    .large_image("icon-new")
                    .large_text("Tataku")
                )
            ) {
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