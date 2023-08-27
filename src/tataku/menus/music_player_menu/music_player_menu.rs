use crate::prelude::*;


pub struct MusicPlayerMenu {
    visualization: BarVisualization,

    settings: SettingsHelper,
    window_size: Arc<WindowSize>,
    song_display: CurrentSongDisplay,
    new_map_helper: LatestBeatmapHelper,

    music_box: MusicBox,
    media_controls: MediaControlHelper,
    event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,
    event_receiver: AsyncUnboundedReceiver<MediaControlHelperEvent>,
}
impl MusicPlayerMenu {
    pub async fn new() -> Self {
        let window_size = WindowSize::get();
        let (event_sender, event_receiver) = async_unbounded_channel();

        Self {
            visualization: BarVisualization::new(0, 1024, Bounds::new(Vector2::ZERO, window_size.0)).await,
            music_box: MusicBox::new(event_sender.clone()).await,
            media_controls: MediaControlHelper::new(event_sender.clone()),

            event_sender, 
            event_receiver,


            settings: SettingsHelper::new(),
            window_size,
            song_display: CurrentSongDisplay::new(),
            new_map_helper: LatestBeatmapHelper::new(),
        }
    }

    fn update_online() {
        tokio::spawn(async move {
            let Some(map) = BEATMAP_MANAGER.read().await.current_beatmap.clone() else { return };

            if let Some(song) = AudioManager::get_song().await {
                OnlineManager::set_action(SetAction::Listening { 
                    artist: map.artist.clone(), 
                    title: map.title.clone(),
                    elapsed: song.get_position(),
                    duration: song.get_duration()
                }, None);
            }
        });
    }

    async fn next(&mut self, game: &mut Game) {
        let mut manager = BEATMAP_MANAGER.write().await;

        if manager.next_beatmap(game).await {
            Self::update_online();
        } else {
            trace!("no next");
        }
    }
    async fn previous(&mut self, game: &mut Game) {
        let mut manager = BEATMAP_MANAGER.write().await;
        
        if manager.previous_beatmap(game).await {
            Self::update_online();
        } else {
            trace!("no prev");
        }
    }

}

#[async_trait]
impl AsyncMenu<Game> for MusicPlayerMenu {
    fn get_name(&self) -> &str {"music_player"}

    async fn on_change(&mut self, into:bool) {
        if into {
            // update our window size
            self.window_size_changed(WindowSize::get()).await;
            self.new_map_helper.update();

            self.visualization.reset();

            // play song if it exists
            if let Some(song) = AudioManager::get_song().await {
                // reset any time mods
                song.set_rate(1.0);
                // // play
                // song.play(true).unwrap();
            }

            // update online to what song we're listening to
            Self::update_online();
        } else {
            debug!("leaving main menu");
        }
    }

    async fn update(&mut self, g:&mut Game) {
        self.settings.update();
        self.song_display.update();

        let mut song_done = false;

        // run updates on the interactables
        self.music_box.update();


        match AudioManager::get_song().await {
            Some(song) => {
                let elapsed = song.get_position();
                let state = if song.is_stopped() {
                    MediaPlaybackState::Stopped
                } else if song.is_paused() {
                    MediaPlaybackState::Paused(elapsed)
                } else if song.is_playing() {
                    MediaPlaybackState::Playing(elapsed)
                } else {
                    //  ??
                    unreachable!()
                };

                self.music_box.update_song_time(elapsed);
                self.music_box.update_song_paused(song.is_paused());
                self.media_controls.update(state, self.settings.integrations.media_controls).await;

                if let Ok(event) = self.event_receiver.try_recv() {
                    match event {
                        MediaControlHelperEvent::Play => song.play(false),
                        MediaControlHelperEvent::Pause => song.pause(),
                        MediaControlHelperEvent::Stop => song.stop(),
                        MediaControlHelperEvent::Toggle => {
                            if song.is_stopped() { 
                                song.play(true); 
                            } else if song.is_playing() {
                                song.pause()
                            } else if song.is_paused() {
                                song.play(false);
                            }
                        }
                        MediaControlHelperEvent::Next     => self.next(g).await,
                        MediaControlHelperEvent::Previous => self.previous(g).await,
                        MediaControlHelperEvent::SeekForward => song.set_position(elapsed + 100.0),
                        MediaControlHelperEvent::SeekBackward => song.set_position(elapsed - 100.0),
                        MediaControlHelperEvent::SeekForwardBy(amt) => song.set_position(elapsed + amt),
                        MediaControlHelperEvent::SeekBackwardBy(amt) => song.set_position(elapsed - amt),
                        MediaControlHelperEvent::SetPosition(pos) => song.set_position(pos),
                        // MediaControlHelperEvent::OpenUri(_) => todo!(),
                        // MediaControlHelperEvent::Raise => todo!(),
                        // MediaControlHelperEvent::Quit => todo!(),
                        _ => {}
                    }
                    
                }

                if !song.is_playing() && !song.is_paused() {
                    song_done = true;
                }
            }
            _ => song_done = true,
        }

        if song_done {
            trace!("song done");
            // this needs to be separate or it double locks for some reason
            let map = BEATMAP_MANAGER.read().await.random_beatmap();

            // it should?
            if let Some(map) = map {
                BEATMAP_MANAGER.write().await.set_current_beatmap(g, &map, false).await;
                Self::update_online();
            }
        }


        self.visualization.update().await;
    }

    async fn draw(&mut self, list: &mut RenderableCollection) {
        // draw visualization
        self.visualization.draw(Vector2::ZERO, list).await;

        // draw interactables
        self.music_box.draw(Vector2::ZERO, list);

        // song info
        self.song_display.draw(list);
    }

    async fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, _game:&mut Game) {
        self.music_box.on_click(pos, button, mods);
    }

    async fn on_mouse_move(&mut self, pos:Vector2, _game: &mut Game) {
        self.music_box.on_mouse_move(pos);
    }

    async fn on_key_press(&mut self, key:Key, game:&mut Game, mods:KeyModifiers) {
        if !mods.alt {
            match key {
                Key::Left => self.previous(game).await,
                Key::Right => self.next(game).await,
                _ => {}
            }
        }
    }

    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size.clone();
        self.music_box = MusicBox::new(self.event_sender.clone()).await;
    }
}

#[async_trait]
impl ControllerInputMenu<Game> for MusicPlayerMenu {}
