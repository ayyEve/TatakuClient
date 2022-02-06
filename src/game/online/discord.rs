// use discord_rpc_client::Client;
use rustcord::{Rustcord, EventHandlers, User, RichPresenceBuilder};
use crate::prelude::*;

const APP_ID:&'static str = "857981337423577109";

lazy_static::lazy_static! {
    static ref CONTINUE_CALLBACKS:AtomicBool = AtomicBool::new(true);
}

pub struct Discord {
    rustcord: Arc<Rustcord>
}

impl Discord {
    pub fn new() -> TatakuResult<Self> {
        match Rustcord::init::<Self>(APP_ID, true, None) {
            Ok(rustcord) => {
                let rustcord = Arc::new(rustcord);

                // setup thread for handling discord callbacks
                let clone = rustcord.clone();
                tokio::spawn(async move {
                    if !CONTINUE_CALLBACKS.load(SeqCst) {return}
                    clone.run_callbacks();
                    // sleep?
                });

                Ok(Self {rustcord})
            }
            Err(e) => {
                Err(TatakuError::String(format!("[Discord] Error starting Discord: {}", e)))
            }
        }
    }

    pub fn change_status(&mut self, desc:String) {
        let presence = RichPresenceBuilder::new()
            .state("Tataku")
            .details(&desc)
            .large_image_key("icon-new")
            .large_image_text("Tataku")
            // .small_image_key("amethyst")
            // .small_image_text("Amethyst")
            .build();

        if let Err(e) = self.rustcord.update_presence(presence) {
            println!("Error updating discord presence: {}", e);
        }
    }
}

// pub enum DiscordError {
//     Whatever
// }

impl EventHandlers for Discord {
    fn ready(user: User) {
        println!("[Discord] Connected as {}#{}", user.username, user.discriminator);
    }
    fn errored(code: i32, message: &str) {
        println!("[Discord] Error: {} (code {})", message, code);
    }
    fn disconnected(code: i32, message: &str) {
        println!("[Discord] Disconnected: {} (code {})", message, code);
    }
}