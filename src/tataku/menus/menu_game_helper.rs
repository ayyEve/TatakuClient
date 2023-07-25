use crate::prelude::*;



pub struct MenuGameHelper {
    pub current_beatmap: CurrentBeatmapHelper,
    current_playmode: CurrentPlaymodeHelper,
    current_mods: ModManagerHelper,
    settings: SettingsHelper,
    pub manager: Option<IngameManager>,

    pub fit_to: Option<(Vector2, Vector2)>,

    /// use bg game settings, or global gamemode?
    use_global_playmode: bool,
    apply_rate: bool,
    check_enabled: Box<dyn Fn(&Settings) -> bool + Send + Sync>,

    loader: Option<AsyncLoader<TatakuResult<IngameManager>>>
}
impl MenuGameHelper {
    pub fn new(use_global_playmode: bool, apply_rate: bool, check_enabled: Box<dyn Fn(&Settings) -> bool + Send + Sync>) -> Self {
        Self {
            current_beatmap: CurrentBeatmapHelper::new(),
            current_playmode: CurrentPlaymodeHelper::new(),
            current_mods: ModManagerHelper::new(),
            settings: SettingsHelper::new(),
            manager: None,
            fit_to: None,
            use_global_playmode,
            apply_rate,
            loader: None,
            check_enabled
        }
    }

    pub async fn setup(&mut self) {
        self.current_playmode.update();
        self.current_beatmap.update();
        self.current_mods.update();
        self.settings.update();


        if !(self.check_enabled)(&self.settings) { return }

        // if !self.settings.background_game_settings.beatmap_select_enabled { return }

        let settings = self.settings.background_game_settings.clone();
        // if !settings.main_menu_enabled { return }

        let map = match &self.current_beatmap.0 {
            Some(map) => map,
            None => return trace!("manager no map")
        };

        let mode = if self.use_global_playmode { self.current_playmode.as_ref().0.clone() } else { settings.mode.clone() };

        if self.loader.is_some() {
            // stop it somehow?
        }

        let map = map.clone();
        let f = async move {manager_from_playmode(mode, &map).await};
        self.loader = Some(AsyncLoader::new(f));

        // match manager_from_playmode(mode, &map).await {
        //     Ok(mut manager) => {
        //         manager.make_menu_background();

        //         if let Some((pos, size)) = self.fit_to {
        //             manager.gamemode.fit_to_area(pos, size).await
        //         }
                
        //         manager.start().await;
        //         trace!("manager started");

        //         self.manager = Some(manager);
        //         // self.visualization.song_changed(&mut self.background_game);
        //     },
        //     Err(e) => {
        //         // self.visualization.song_changed(&mut None);
        //         NotificationManager::add_error_notification("Error loading beatmap", e).await;
        //     }
        // }

    }

    pub async fn update(&mut self) {
        if self.settings.update() {
            if let Some(manager) = &mut self.manager {
                manager.force_update_settings().await;
            }
        }

        if self.current_mods.update() {
            if let Some(manager) = &mut self.manager {
                manager.apply_mods(self.current_mods.as_ref().clone()).await;
            }
        }

        let mut refresh_map = self.current_playmode.update();
        refresh_map |= self.current_beatmap.update();
        if refresh_map { self.setup().await; }


        if let Some(manager) = &mut self.manager {
            manager.update().await;
            
            if manager.completed {
                manager.on_complete();
                self.manager = None;
            }
        }

        if let Some(loader) = &self.loader {
            if let Some(result) = loader.check().await {
                self.loader = None;
                
                match result {
                    Ok(mut manager) => {
                        manager.make_menu_background();

                        if let Some((pos, size)) = self.fit_to {
                            manager.fit_to_area(Bounds::new(pos, size)).await
                        }
                        
                        manager.start().await;
                        self.manager = Some(manager);
                    },
                    Err(e) => NotificationManager::add_error_notification("Error loading beatmap", e).await,
                }
            }
        }
        
        match AudioManager::get_song().await {
            Some(song) => {
                if !song.is_playing() && !song.is_paused() {
                    // restart the song at the preview point
                    if let Some(map) = &self.current_beatmap.clone().0 {
                        let _ = song.set_position(map.audio_preview);
                        if self.apply_rate { song.set_rate(self.current_mods.get_speed()); }
                        
                        song.play(false);
                        self.setup().await;
                    }
                }
            }

            // no value, set it to something
            _ => {
                if let Some(map) = &self.current_beatmap.clone().0 {
                    let audio = AudioManager::play_song(map.audio_filename.clone(), true, map.audio_preview).await.unwrap();
                    if self.apply_rate { audio.set_rate(self.current_mods.get_speed()); }
                }
            },
        }

    }

    pub async fn draw(&mut self, list: &mut RenderableCollection) {
        if let Some(manager) = &mut self.manager {
            manager.draw(list).await;
        }
    }

    pub async fn key_down(&mut self, key:Key, mods:KeyModifiers) {
        if let Some(manager) = &mut self.manager {
            manager.key_down(key, mods).await
        }
    }

    pub async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        if let Some(manager) = &mut self.manager {
            manager.window_size_changed(window_size).await
        }
    }

    pub async fn fit_to_area(&mut self, pos: Vector2, size: Vector2) {
        self.fit_to = Some((pos, size));

        if let Some(manager) = &mut self.manager {
            manager.fit_to_area(Bounds::new(pos, size)).await
        }
    } 
}

impl Drop for MenuGameHelper {
    fn drop(&mut self) {
        if let Some(manager) = &mut self.manager {
            manager.on_complete()
        }
    }
}
