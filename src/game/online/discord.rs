// use discord_rpc_client::Client;

const _APP_ID:u64 = 857981337423577109;

pub struct Discord {
    // client: Client
}

impl Discord {
    pub fn new() -> Self {

        {
            // // Create the client
            // let mut drpc = Client::new(425407036495495169);

            // // Start up the client connection, so that we can actually send and receive stuff
            // drpc.start();

            // // Set the activity
            // drpc.set_activity(|act| act.state("test"))
            //     .expect("Failed to set activity");

            // Self{client:drpc}
        }

        // let mut client = Client::new(APP_ID);
        // client.start();

        // Self{client}
        Self{}
    }

    pub fn change_status(&mut self, _desc:String) {
        // let presence;
        // let desc;
        // match mode {
        //     GameMode::Closing => return, // dispose of anything?
        //     GameMode::None => desc = "idle".to_owned(), // idle
        //     GameMode::Ingame(beatmap) => {desc = format!("Playing {}", beatmap.lock().unwrap().metadata.version_string())}, // playing map
        //     GameMode::InMenu(_menu) => desc = "In a menu".to_owned(), // in a menu (idle?)
        //     GameMode::Replaying(_map, _replay, _) => desc = format!("Spectating").clone(), // let username = replay.score.username.clone();
        // };


        // let ugh = self.client.set_activity(|a|
        //     a
        //     .state("Taiko.rs")
        //     .details(desc)
        //     .assets(|assets|
        //         assets
        //         .large_image("icon")
        //         .large_text("Taiko.rs")
        //     )
        // );
        // match ugh {
        //     Ok(thing) => println!("discord update successful: {:?}", thing), 
        //     Err(e) => println!("discord oof: {}", e),
        // }

        // if let Err(e) = self.client.update_presence(presence) {
        //     println!("Error updating discord presence: {}", e);
        // }


    }
}

// pub enum DiscordError {
//     Whatever
// }

// impl EventHandlers for Discord {
//     fn ready(user: User) {
//         println!("[Discord] User {}#{} logged in...", user.username, user.discriminator);
//     }
//     fn errored(code: i32, message: &str) {
//         println!("[Discord] Error: {} (code {})", message, code);
//     }
//     fn disconnected(code: i32, message: &str) {
//         println!("[Discord] Disconnected: {} (code {})", message, code);
//     }
// }