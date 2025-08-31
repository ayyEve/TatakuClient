use tataku_engine::prelude::*;
use discord_rich_presence::{
    DiscordIpc, 
    DiscordIpcClient,
    activity::{ 
        Assets, 
        Timestamps, 
        Activity
    }, 
};

const APP_ID:&str = "857981337423577109";
const RECONNECT_INTERVAL: f32 = 5_000.0; // every 5 seconds try again

pub struct Discord {
    client: DiscordIpcClient,
    last_connection_attempt: Option<TatakuInstant>,

    enabled: bool,
    connected: bool,
}
impl Discord {
    fn build() -> TatakuResult<Box<dyn TatakuIntegration>> {
        Ok(Box::new(Self {
            client: DiscordIpcClient::new(APP_ID).map_err(DiscordError)?,
            connected: false,
            enabled: false,
            last_connection_attempt: None,
        }))
    }

    pub fn builder() -> TatakuIntegrationBuilder {
        TatakuIntegrationBuilder {
            name: "Discord",
            build: Self::build
        }
    }

    /// attempt to reconnect
    fn reconnect(&mut self) -> TatakuResult {
        // dont connect if we aren't enabled, or if we're already connected
        if !self.enabled || self.connected { return Ok(()) }


        // make sure we wait at least RECONNECT_INTERVAL before trying to reconnect
        if let Some(last_check) = self.last_connection_attempt
        && last_check.as_millis() < RECONNECT_INTERVAL { 
            return Ok(()) 
        }
        self.last_connection_attempt = Some(TatakuInstant::now());

        // attempt to reconnect
        self.client.connect().map_err(DiscordError)?;

        // should be connected at this point
        self.connected = true;
        
        Ok(())
    }
}

impl TatakuIntegration for Discord {
    fn name(&self) -> CowStr { Cow::Borrowed("Discord") }
    fn init(
        &mut self, 
        #[cfg(feature="graphics")]
        _window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> TatakuResult<()> {
        Ok(())
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
        event: &TatakuIntegrationEvent,
        values: &dyn Reflect,
        _actions: &mut ActionQueue,
    ) {
        if !self.enabled || !self.connected { return }

        let Ok(username) = values
            .reflect_get::<String>("global.username")
            .map(|u| u.cloned())
            else { return };


        let mut activity = Activity::new();

        let mut assets = Assets::new()
            .large_image("icon-new")
            .large_text(&username); // TODO: make the username of the logged-in user

        match event {
            TatakuIntegrationEvent::BeatmapStarted { 
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

                let infos = values
                    .reflect_get::<GamemodeInfos>("global.infos")
                    .unwrap();

                let playmode_display = infos
                    .get_info(playmode)
                    .map_or_else(
                        |_| playmode.to_owned(), 
                        |i| i.display_name.to_owned().into()
                    );

                assets = assets
                    .small_image("icon") // TODO: use a url for the image, where if it doesnt exist, it gives some default, so we always have the mode text
                    .small_text(playmode_display.to_string()); //values.global.gamemode_infos.get_info(playmode).map(|a| a.display_name.to_owned()).unwrap_or(playmode.to_owned()));

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
            TatakuIntegrationEvent::SongChanged { 
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
                    .state(format!("Listening to {artist} - {title}"))
                ;
            }
            TatakuIntegrationEvent::BeatmapEnded => {
                activity = activity
                    .state("Idle")
                ;
            }

            _ => return
        }

        if let Err(e) = self.client.set_activity(activity.assets(assets)) {
            error!("error setting discord presence: {e:?}");
        }

    }
}
impl Drop for Discord {
    fn drop(&mut self) {
        let _ = self.client.close();
    }
}

struct DiscordError(Box<dyn std::error::Error>);
impl From<DiscordError> for TatakuError {
    fn from(value: DiscordError) -> Self {
        Self::String(value.0.to_string())
    }
}
