use crate::prelude::*;
use super::GameState;

// action handlers. here bc they're so big
impl Game {

    #[cfg(feature="graphics")] 
    fn next_gameplay_id(&self) -> GameplayId {
        Arc::new(self.gameplay_managers
            .keys()
            .max()
            .map(|a| **a + 1)
            .unwrap_or_default()
        )
    }

    // #[cfg(feature="graphics")]
    // fn handle_previous_menu(&mut self, current_menu: &str)  {
    //     let in_multi = self.multiplayer_manager.is_some();
    //     let in_spec = self.spectator_manager.is_some();

    //     if in_multi { 
    //         return self.handle_custom_menu("lobby_menu", None);
    //     }
    //     if in_spec { 
    //         return self.handle_custom_menu("beatmap_select", None); 
    //     }

    //     match current_menu {
    //         // score menu with no multi or spec is the beatmap select menu
    //         "score_menu" => self.handle_custom_menu("beatmap_select", None), 

    //         // beatmap menu with no multi or spec is the main menu
    //         "beatmap_select" => self.handle_custom_menu("main_menu", None),

    //         _ => {
    //             error!("unhandled previous menu request for menu {current_menu}");
    //         }
    //     }
    // }


    pub(super) fn handle_actions(&mut self, actions: Option<Vec<TatakuAction>>) {
        if let Some(actions) = actions {
            self.actions.extend(actions);
        }

        for action in self.actions.take() {
            self.handle_action(action);
        }
    }

    #[cfg(feature="graphics")]
    pub(super) fn handle_custom_menu(
        &mut self, 
        id: impl ToString,
    ) {
        let selector = (
            id.to_string().into(), 
            CustomMenuSource::Any
        );

        // let menu = self.custom_menus.iter().rev().find(|cm| cm.id == id);
        if let Some(menu) = self.custom_menu_manager.get_menu(selector) {
            let menu = menu.build(&mut self.values);

            self.queue_state_change(
                GameState::SetMenu(Box::new(menu))
            );
        } else {
            let id = id.to_string();
            match &*id {
                "none" => {}
                "main_menu" 
                    => panic!("Main menu could not be loaded. did eve fuck up the main_menu.xml?"),
                _ => {
                    error!("custom menu not found! {id}, going to main menu instead");
                    self.actions.push(MenuAction::set_menu("main_menu").into());
                }
            }
        }
    }

    #[cfg(feature="graphics")]
    pub(super) fn handle_custom_dialog(
        &mut self, 
        id: impl ToString, 
        options: DialogCreateOptions,
    ) {
        let id:ArcStr = id.to_string().into();
        let Some(dialog) = self.custom_menu_manager
            .get_dialog((id.clone(), CustomMenuSource::Any))
        else {
            // if id == "settings" {
            //     self.ui_manager.add_dialog(
            //         Box::new(SettingsMenu::new(&self.values.settings)), 
            //         SettingsMenu::DEFAULT_OPTIONS,
            //         &mut self.values, 
            //         &mut self.actions,
            //     );
            // } else {
                error!("unknown dialog id: {id}");
            // }

            return;
        };

        let options = DialogCreateOptions::merge(
            options,
            dialog.options(),
        );

        let dialog = dialog.build(&mut self.values);

        self.ui_manager.add_dialog(
            Box::new(dialog),
            options,
            &mut self.values,
            &mut self.actions,
            &mut self.text_layout_contexts,
        );
    }



    #[cfg(feature="graphics")]
    pub(super) fn handle_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::SetMenu { id } => self.handle_custom_menu(id),

            // MenuAction::PreviousMenu(current_menu) 
            //     => self.handle_previous_menu(&current_menu),

            MenuAction::AddDialog {
                id, 
                options, 
            } => self.handle_custom_dialog(id.to_string(), *options),

            MenuAction::AddDialogRaw {
                dialog, 
                options
            } => {
                self.ui_manager.add_dialog(
                    dialog, 
                    *options, 
                    &mut self.values, 
                    &mut self.actions,
                    &mut self.text_layout_contexts,
                );
            }
        }
    }

    pub(super) fn handle_song_action(&mut self, action: SongAction) {
        match action {
            // needs to be before trying to get the audio because audio might be none when this is run
            SongAction::Set(action) => {
                if let Err(e) = self.song_manager.handle_song_set_action(
                    action, 
                    &mut self.actions, 
                    &mut self.audio_manager,
                    &self.values.settings
                ) {
                    error!("Error handling SongMenuSetAction: {e:?}");
                }

                let Some(audio) = self.song_manager.instance() 
                else { return };
                
                let Some(current) = self.beatmap_manager
                    .current_beatmap()
                else { return };

                self.actions.push(TatakuIntegrationEvent::SongChanged { 
                    artist: current.artist.clone(), 
                    title: current.title.clone(), 
                    image_path: current.image_filename.clone(), 
                    elapsed: audio.get_position(), 
                    duration: audio.get_duration()
                }.into());
            }
            #[cfg(feature="graphics")] 
            SongAction::HookFFT(hook) => self
                .song_manager
                .hook_fft(hook),
            
            other => {
                let Some(audio) = self.song_manager.instance() 
                else { return };

                match other {
                    SongAction::Play => audio.play(false),
                    SongAction::Restart => audio.play(true),
                    SongAction::Pause => audio.pause(),
                    SongAction::Stop => audio.stop(),
                    SongAction::Toggle if audio.is_playing() => audio.pause(),
                    SongAction::Toggle => audio.play(false),
                    SongAction::SeekBy(seek) 
                        => audio.set_position(audio.get_position() + seek),
                    SongAction::SetPosition(pos) => audio.set_position(pos),
                    SongAction::SetRate(rate) => audio.set_rate(rate),
                    SongAction::SetVolume(vol) => audio.set_volume(vol),
                    
                    // handled above
                    _other => unreachable!()
                }
            }
        }

        // update song state
        self.values.song.update(self.song_manager.instance());
    }

    pub(super) fn handle_mod_action(&mut self, action: ModAction) {
        let mods = &mut self.values.global.mods;
        match action {
            ModAction::AddMod(mod_name) => mods.add_mod(mod_name).nope(),
            ModAction::RemoveMod(mod_name) => mods.remove_mod(mod_name),
            ModAction::ToggleMod(mod_name) => mods.toggle_mod(mod_name).nope(),
            ModAction::SetSpeed(speed) => mods.set_speed(speed),
            ModAction::AddSpeed(speed) => mods.set_speed(mods.get_speed() + speed),
            ModAction::SetMods(new_mods) => mods.mods = new_mods,
        }
        self.values.global.update_mods();

        // update the song's rate
        self.actions.push(SongAction::SetRate(
            self.values.global.mods.get_speed()
        ).into());

        // apply mods to all gameplay managers
        #[cfg(feature="graphics")] 
        for (m, i) in self.gameplay_managers.values_mut() {
            if i.mods.is_some() { continue }
            m.apply_mods(self.values.global.mods.clone());
        }

        // update the beatmap groupings to update the diffs
        let mods = self.global.mods.clone();
        let playmode = self.global.playmode.clone();
        let sort_by = self.values.settings.sort_by;
        self.values.beatmap_manager.apply_filter(
            &mods, 
            &playmode, 
            sort_by,
            &mut self.difficulty_manager,
        );
    }

    pub(super) fn handle_beatmap_action(&mut self, action: BeatmapAction) {
        match action {
            #[cfg(feature="gameplay")]
            BeatmapAction::PlaySelected => {
                let Some(map) = self
                    .values.values
                    .beatmap_manager
                    .current_beatmap()
                    .cloned()
                else { return };

                let mods = self.global.mods.clone();
                let mode = self.global.playmode.clone();

                match manager_from_playmode(
                    &self.global.gamemode_infos,
                    &mode, 
                    &map, 
                    mods.clone(),
                    &self.settings,
                ) {
                    Ok(mut manager) => {
                        let start_time = manager.start_time as u64;

                        manager.handle_action(
                            GameplayAction::ApplyMods(mods), 
                            &self.settings
                        );

                        let multiplayer = self.multiplayer_manager
                            .as_ref()
                            .map(|a| a.lobby.id)
                            .and_then(|i| self.online_manager.lobby(i).cloned())
                            ;

                        self.handle_event(TatakuIntegrationEvent::BeatmapStarted { 
                            start_time, 
                            beatmap: map, 
                            playmode: mode, 
                            multiplayer, 
                            spectator: self.spectator_manager
                                .as_ref()
                                .map(|s| s.host_username.clone())
                        });
                        
                        self.queue_state_change(
                            GameState::Ingame(Box::new(manager))
                        );
                    }
                    Err(e) => self.actions.push(Notification::new_error(
                        "Error loading beatmap", 
                        e
                    ).into()),
                }
            }

            #[cfg(feature="gameplay")]
            BeatmapAction::ConfirmSelected => {
                if let Some(multi) = &mut self.multiplayer_manager {
                    // go back to the lobby before any checks
                    // this way if for some reason something down below fails, the user is in the lobby and not stuck in limbo
                    #[cfg(feature="graphics")] 
                    self.actions.push(MenuAction::set_menu("lobby_menu").into());

                    if !multi.is_host() { 
                        return warn!("trying to set lobby beatmap while not the host ??");
                    };

                    let Some(map) = self
                        .values.values
                        .beatmap_manager
                        .current_beatmap() 
                    else { return };

                    let playmode = &self.values.values.global.playmode;
                    self.values.values.online_manager.update_lobby_beatmap(
                        map, 
                        playmode.to_string()
                    );

                } else if let Some(spec_man) = self
                    .spectator_manager
                    .as_mut() 
                {
                    let Some(map) = self
                        .values.values
                        .beatmap_manager
                        .current_beatmap() 
                    else { return };

                    self.values.values.online_manager.handle_action(OnlineAction::ChatAction(
                        ChatAction::SendMessage { 
                            channel: spec_man.host_username.to_string(), 
                            message: BeatmapLink {
                                beatmap_hash: map.beatmap_hash.to_string(),
                                beatmap_title: map.version_string(),
                                download_link: None,
                            }.to_string()
                        }
                    ));
                } else {
                    // play map
                    self.handle_beatmap_action(BeatmapAction::PlaySelected);
                }
            }

            BeatmapAction::Set(
                hash, 
                options
            ) => self.handle_beatmap_action(BeatmapAction::SetFromHash(
                hash, 
                options
            )),
            
            BeatmapAction::SetFromHash(hash, options) => {
                if self.beatmap_manager.has_hash(&hash) {
                    let config = self.create_select_beatmap_config(
                        options.restart_song,
                        options.use_preview_point,
                    );
                    self.set_current_beatmap(hash, config);

                    return;
                }

                #[cfg(feature="gameplay")]
                if self.multiplayer_manager.is_some() {
                    // if we're in a multiplayer lobby, and the map doesnt exist, remove the map
                    self.handle_beatmap_action(BeatmapAction::Remove);
                    return
                }

                match options.if_none {
                    MapActionIfNone::ContinueCurrent => {},
                    MapActionIfNone::SetNone 
                        => self.handle_beatmap_action(BeatmapAction::Remove), 
                    MapActionIfNone::Random(preview) => {
                        let Some(map) = self
                            .beatmap_manager
                            .random_beatmap() 
                        else { return };

                        self.handle_beatmap_action(BeatmapAction::SetFromHash(
                            map, 
                            options.use_preview_point(preview)
                        ));
                    }
                }
            }

            BeatmapAction::SetPlaymode(new_mode) 
                => self.update_playmode(&new_mode),

            BeatmapAction::Random(use_preview) => {
                let Some(hash) = self
                    .beatmap_manager
                    .random_beatmap() 
                else { return };

                let config = self.create_select_beatmap_config(
                    true,
                    use_preview
                );
                self.set_current_beatmap(hash, config);
            }
            BeatmapAction::Remove => {
                self.remove_current_beatmap();
                // warn!("removing beatmap");
                #[cfg(feature="graphics")] {
                    self.background_image = None;
                }
            }

            BeatmapAction::Delete(hash) => {
                let config = self.create_select_beatmap_config(
                    true, true
                );

                self.delete_beatmap(
                    hash,
                    PostDelete::Next,
                    config,
                );
            }
            BeatmapAction::DeleteCurrent(post_delete) => {
                let Some(map_hash) = self.beatmap_manager.current_beatmap
                else { return };

                let config = self.create_select_beatmap_config(
                    true, 
                    true
                );
                
                self.delete_beatmap(
                    map_hash,
                    post_delete,
                    config,
                );
            }
            BeatmapAction::Next => {
                let config = self.create_select_beatmap_config(
                    true, 
                    false
                );

                self.next_beatmap(config);
            }
            BeatmapAction::Previous(if_none) => {
                let mut config = self.create_select_beatmap_config(
                    true, 
                    false
                );

                if self.previous_beatmap(config.clone()) { return }

                // no previous map availble, handle accordingly
                match if_none {
                    MapActionIfNone::ContinueCurrent => return,
                    MapActionIfNone::Random(use_preview) => {
                        config.use_preview_time = use_preview;

                        let Some(hash) = self
                            .beatmap_manager
                            .random_beatmap() 
                        else { return };

                        self.set_current_beatmap(hash, config);
                    }
                    MapActionIfNone::SetNone 
                        => self.remove_current_beatmap(),
                }
            }

            BeatmapAction::InitializeManager => {
                self.values.values.beatmap_manager.initialize(
                    self.values.values.settings.sort_by, 
                    &self.values.values.global.mods, 
                    &self.values.values.global.playmode, 
                    &mut self.difficulty_manager,
                );
            }
            BeatmapAction::AddBeatmap { 
                map, 
                add_to_db 
            } => {
                self.beatmap_manager.add_beatmap(&map, add_to_db);

                if self.beatmap_manager.initialized {
                    self.values.values.beatmap_manager.refresh_maps(
                        &self.values.values.global.mods, 
                        &self.values.values.global.playmode, 
                        self.values.values.settings.sort_by,
                        &mut self.difficulty_manager,
                    );
                }
            }

            // beatmap list actions
            BeatmapAction::ListAction(list_action) => {
                match list_action {
                    BeatmapListAction::Refresh => {
                        self.values.values.beatmap_manager.refresh_maps(
                            &self.values.values.global.mods, 
                            &self.values.values.global.playmode, 
                            self.values.values.settings.sort_by,
                            &mut self.difficulty_manager,
                        );
                    }

                    BeatmapListAction::ApplyFilter { 
                        filter
                    } => {
                        self.beatmap_manager.filter_text = filter.unwrap_or_default();
                        self.values.values.beatmap_manager.apply_filter(
                            &self.values.values.global.mods,
                            &self.values.values.global.playmode,
                            self.values.values.settings.sort_by,
                            &mut self.difficulty_manager
                        );
                    }
                    BeatmapListAction::NextMap => self.beatmap_manager.next_map(),
                    BeatmapListAction::PrevMap => self.beatmap_manager.prev_map(),
                    BeatmapListAction::NextSet => self.beatmap_manager.next_set(),
                    BeatmapListAction::PrevSet => self.beatmap_manager.prev_set(),

                    BeatmapListAction::SelectSet(set_id) 
                        => self.beatmap_manager.select_set(set_id),
                }
            }

            #[cfg(not(feature="gameplay"))]
            _ => {}
        }

        // handle beatmap manager actions
        let bm_actions = self.beatmap_manager.actions.take();
        for i in bm_actions {
            self.handle_action(i);
        }
    }

    #[cfg(feature="graphics")] 
    pub(super) fn handle_current_game_action(&mut self, action: CurrentGameAction) {
        if matches!(action, CurrentGameAction::Pause {..}) {

            let CurrentGameAction::Pause { 
                id, 
            } = action else { unreachable!() };
            
            if !self.current_state.is_ingame() {
                warn!("got pause for current gameplay but no current gameplay");
                return
            }

            let GameState::Ingame(gameplay) = self
                .current_state
                .take() 
            else { unreachable!() };

            self.current_state = GameState::None;
            self.handle_menu_action(MenuAction::SetMenu {
                id: id.into(),
            });
            
            // make sure it has the latest window size
            self.pending_gameplay_manager = Some(gameplay);
            
            return;
        }

        let Some(mut manager) = self.pending_gameplay_manager
            .take() 
        else { 
            warn!("Got action {action:?} but no pending gameplay manager");
            return 
        };

        match action {
            CurrentGameAction::Start => {
                manager.start();
                #[cfg(feature="gameplay")]
                self.queue_state_change(GameState::Ingame(manager));
            }
            CurrentGameAction::Resume => {
                #[cfg(feature="gameplay")]
                self.queue_state_change(GameState::Ingame(manager));
            }
            CurrentGameAction::Restart => {
                manager.reset();
                #[cfg(feature="gameplay")]
                self.queue_state_change(GameState::Ingame(manager));
            }
            CurrentGameAction::Free => {
                #[cfg(feature="graphics")] 
                manager.cleanup_textures(&mut self.skin_manager);
            }

            CurrentGameAction::Pause { .. } => unreachable!(),
        }
    }
    
    pub(super) fn handle_game_action(&mut self, action: GameAction) {
        match action {
            #[cfg(feature="gameplay")]
            GameAction::Quit => self.queue_state_change(GameState::Closing),

            #[cfg(feature="gameplay")]
            GameAction::RestartOnline => self.init_online(),

            #[cfg(feature="graphics")] 
            GameAction::CurrentGameAction(action) 
                => self.handle_current_game_action(action),

            #[cfg(feature="gameplay")]
            GameAction::WatchReplay(score) => {
                let map = score.beatmap_hash;
                let mode = &score.playmode;

                let Some(beatmap) = self.beatmap_manager
                    .get_by_hash(&map) 
                else {
                    self.actions.push(
                        Notification::default()
                        .text("You don't have that map!")
                        .duration(5000.0)
                        .color(Color::RED)
                        .into()
                    );
                    return;
                };

                let mods = self.values.global.mods.clone();

                match manager_from_playmode_path_hash(
                    &self.global.gamemode_infos,
                    mode, 
                    &beatmap.file_path, 
                    beatmap.beatmap_hash, 
                    mods, 
                    &self.settings
                ) {
                    Ok(mut manager) => {
                        manager.set_mode(GameplayMode::Replay(score).into());
                        self.queue_state_change(GameState::Ingame(Box::new(
                            manager
                        )));
                    }
                    Err(e) => self.actions.push(
                        Notification::new_error(
                            "Error loading beatmap", 
                            e
                        ).into()
                    ),
                }
            }
            GameAction::SetValue(key, value) => {
                let values = self.values.as_dyn_mut();
                let a = format!("{value:?}");

                let r = match value {
                    TatakuValue::F32(n) 
                        => values.reflect_insert(&key, n),
                    TatakuValue::U32(n) 
                        => values.reflect_insert(&key, n),
                    TatakuValue::U64(n) 
                        => values.reflect_insert(&key, n),
                    TatakuValue::Bool(b) 
                        => values.reflect_insert(&key, b),
                    TatakuValue::String(s) 
                        => values.reflect_insert(&key, s),
                    TatakuValue::Reflect(reflect) 
                        => values.impl_insert(
                            ReflectPath::new(&key), 
                            reflect
                        ),
                    
                    other => {
                        warn!("OTHER NOT HANDLED!!! {other:?}");
                        Ok(())
                    }
                };
                if let Err(e) = r {
                    error!("Error updating value '{key}' with {a}: {e:?}");
                }
            }
            #[cfg(feature="graphics")]
            GameAction::ViewScore(score) => {
                if self.beatmap_manager.has_hash(&score.beatmap_hash) {
                    let info = self.values.global
                        .gamemode_infos
                        .get_info(&score.playmode)
                        .copied()
                        .unwrap_or_default();

                    self.set_current_beatmap(
                        score.beatmap_hash, 
                        SelectBeatmapConfig::new(
                            ModManager::new(
                                score.mods.iter(),
                                score.speed, 
                                &info
                            ),
                            score.playmode.clone().into(),
                            false,
                            true
                        ),
                    );

                    self.values.values.score = ReflectScore::new(&score, &info);

                    // show score menu
                    self.values.impl_insert("var.score_menu.allow_retry".into(), Box::new(false)).unwrap();
                    self.handle_custom_menu("score_menu");
                } else {
                    error!("Could not find map from score!");
                }
            }
            #[cfg(feature="graphics")]
            GameAction::HandleMessage(message) 
                => self.ui_manager.add_message(message),

            GameAction::RefreshScores => self.score_manager.force_update = true,
            #[cfg(feature="graphics")] 
            GameAction::ViewScoreId(id) => {
                if let Some(score) = self.score_manager.get_score(id) {
                    self.handle_game_action(GameAction::ViewScore(score.clone()));
                }
            }
            #[cfg(feature="graphics")]
            GameAction::HandleEvent(
                event, 
                param
            ) => self.queued_events.push((event, param)),
            
            #[cfg(feature="graphics")] 
            GameAction::AddNotification(
                notif
            ) => self.notification_manager.add_notification(notif),

            #[cfg(feature="graphics")]
            GameAction::UpdateBackground => {
                let Some(filename) = self.values.current_beatmap_prop(
                    |b| b.image_filename.clone()
                ) 
                else { return };
                
                self.background_image = self.skin_manager.get_texture(
                    &filename, 
                    &TextureSource::Raw, 
                    SkinUsage::Background, 
                    false
                );

                if let Some(i) = &mut self.background_image {
                    i.origin = Vector2::ZERO;
                }

                self.resize_bg();
            },
            #[cfg(feature="graphics")]
            GameAction::CopyToClipboard(text) => { 
                let _ = self.window_proxy.send_event(
                    WindowAction::CopyToClipboard(text)
                ); 
            }

            GameAction::RefreshPlaymodeValues => {
                let playmode = self.global.playmode.clone();
                self.update_playmode(&playmode);
            }
            GameAction::UpdatePlaymodeActual(actual) => {
                self.values.global.update_playmode_actual(actual);
            }

            GameAction::RefreshSkins => {
                let mut list = vec!["None".to_owned()];
                for f in std::fs::read_dir(SKINS_FOLDER).unwrap() {
                    list.push(f.unwrap().file_name().to_string_lossy().to_string());
                }
                self.values.enums.skins = list;
            }


            #[cfg(feature="graphics")]
            GameAction::NewGameplayManager(config) => {
                match match &config {
                    NewManager {
                        mods,
                        map_hash: Some(map_hash),
                        path: Some(path),
                        playmode,
                        ..
                    } => {
                        let playmode = playmode.clone()
                            .unwrap_or_else(|| self.values.global.playmode.clone());
                        let mods = mods.clone()
                            .unwrap_or_else(|| self.values.global.mods.clone());
                        
                        manager_from_playmode_path_hash(
                            &self.global.gamemode_infos,
                            &playmode, 
                            path, 
                            *map_hash, 
                            mods, 
                            &self.settings,
                        )
                    }
                    NewManager {
                        mods,
                        map_hash,
                        playmode,
                        ..
                    } => {
                        let map_hash = map_hash.unwrap_or_else(
                            || self.values.current_beatmap_prop(
                                |b| b.beatmap_hash
                                )
                                .unwrap_or_default()
                        );

                        let Some(meta) = self
                            .beatmap_manager
                            .get_by_hash(&map_hash) 
                        else { return };
                        let playmode = playmode.clone().unwrap_or_else(
                            || self.values.global.playmode_actual.clone()
                        );

                        let mods = mods.clone().unwrap_or_else(
                            || self.values.global.mods.clone()
                        );

                        manager_from_playmode(
                            &self.global.gamemode_infos,
                            &playmode, 
                            &meta, 
                            mods,
                            &self.settings,
                        )
                    }
                } {
                    Ok(mut manager) => {
                        manager.reload_skin(
                            &mut self.skin_manager, 
                            &self.values.settings
                        );

                        if let Some(mode) = config.gameplay_mode.clone() {
                            manager.set_mode(mode.into());
                        }
                        
                        manager.window_size_changed(self.values.game.window_size);
                        
                        if let Some(bounds) = config.area {
                            manager.handle_action(
                                GameplayAction::FitToArea(bounds), 
                                &self.settings
                            );
                        }
                        manager.reset();

                        let id = self.next_gameplay_id();
                        self.ui_manager.add_message(Message::new(
                            config.owner, 
                            "gameplay_manager_create", 
                            MessageValue::Custom(id.clone())
                        ));
                        manager.set_id(id.clone());

                        self.gameplay_managers.insert(id, (manager, config));
                    }

                    Err(e) 
                        => error!("Error creating gameplay manager: {e}"),
                }
            }

            #[cfg(feature="graphics")]
            GameAction::DropGameplayManager(id) => {
                self.gameplay_managers.remove(&id);
            }

            #[cfg(feature="graphics")]
            GameAction::GameplayAction(id, action) => {
                let Some(gameplay) = (if *id == u32::MAX {
                    trace!("gameplay is state");
                    self.current_state
                        .get_ingame()
                        .map(|a| &mut **a)
                } else {
                    self.gameplay_managers
                        .get_mut(&id)
                        .map(|a| &mut a.0)
                })
                else { return };

                
                if let &GameplayAction::RequestDifficulty = &action {
                    gameplay.update_difficulty(&mut self.difficulty_manager);
                } else {
                    gameplay.handle_action(action, &self.values.settings);
                }
            }


            GameAction::UpdateSettings(run) => {
                (run)(&mut self.values.settings);
            }

            #[cfg(not(feature="graphics"))] _ => {} 
        }
    }

    pub(super) fn handle_multiplayer_action(&mut self, action: MultiplayerAction) {
        match action {
            #[cfg(feature="graphics")]
            MultiplayerAction::StartMultiplayer => {
                self.online_manager.add_lobby_listener();
                self.handle_custom_menu("lobby_select");
            },

            #[cfg(feature="graphics")]
            MultiplayerAction::ExitMultiplayer => {
                self.handle_multiplayer_action(MultiplayerAction::LeaveLobby);

                // if ingame, dont change state. this way the user can keep playing the map
                if !self.current_state.is_ingame() {
                    self.handle_custom_menu("main_menu");
                }

                self.online_manager.remove_lobby_listener();
            }

            #[cfg(feature="gameplay")]
            MultiplayerAction::CreateLobby { 
                name, 
                password, 
                private, 
                players 
            } => {
                self.online_manager.multiplayer_data.lobby_creation_pending = true;
                info!("sending create");

                self.online_manager.send_packet(
                    MultiplayerPacket::Client_CreateLobby { 
                        name, 
                        password, 
                        private, 
                        players 
                    }
                );
            }
            #[cfg(feature="gameplay")]
            MultiplayerAction::LeaveLobby => {
                self.multiplayer_manager = None;
                self.online_manager.send_packet(MultiplayerPacket::Client_LeaveLobby);
                #[cfg(feature="graphics")] 
                self.handle_custom_menu("lobby_select");
            }
            #[cfg(feature="gameplay")]
            MultiplayerAction::JoinLobby { lobby_id, password } => {
                self.online_manager.multiplayer_data.lobby_join_pending = true;
                if let Some(multi_manager) 
                    = &mut self.multiplayer_manager 
                {
                    // if we're already in this lobby, dont do anything
                    if multi_manager.lobby.id == lobby_id { return }

                    // otherwise, leave our current lobby
                    self.handle_multiplayer_action(MultiplayerAction::LeaveLobby);
                }

                self.online_manager.send_packet(MultiplayerPacket::Client_JoinLobby { 
                    lobby_id, 
                    password 
                });
            }

            #[cfg(feature="gameplay")]
            MultiplayerAction::SetBeatmap { hash, mode } => {
                let Some(map) = self.beatmap_manager.get_by_hash(&hash) 
                else { return };

                let mode = mode
                    .unwrap_or_else(|| self.values.global.playmode_actual.clone());

                self.online_manager.update_lobby_beatmap(&map, mode.to_string());
            }

            #[cfg(feature="gameplay")]
            MultiplayerAction::InviteUser { user_id } => {
                self.online_manager.invite_user(user_id);
            }

            // lobby actions
            #[cfg(feature="gameplay")]
            MultiplayerAction::LobbyAction(LobbyAction::Leave) => {
                self.handle_multiplayer_action(MultiplayerAction::LeaveLobby);
            }
            #[cfg(feature="gameplay")]
            MultiplayerAction::LobbyAction(action) => {
                let Some(multi_manager) 
                    = &mut self.multiplayer_manager 
                else { return };

                multi_manager.handle_lobby_action(
                    action, 
                    &self.values.settings, 
                    &mut self.actions
                );
            }

            // ignore unhandled messages when either of these features arent enabled
            #[cfg(any(not(feature="gameplay"), not(feature="graphics")))] _ => {}
        }
    }

    #[cfg(feature="gameplay")]
    pub(super) fn handle_multiplayer_packet(
        &mut self, 
        packet: MultiplayerPacket
    ) -> TatakuResult {
        // if we have a multi manager, pass the packet onto it as well
        if let Some(multi_manager) 
            = &mut self.multiplayer_manager 
        {
            let ig_manager = self
                .current_state
                .get_ingame();

            let manager_maybe = multi_manager.handle_packet(
                &mut self.values, 
                &packet, 
                ig_manager,
                &mut self.actions,
            )?;

            if let Some(manager) = manager_maybe {
                // start the manager
                debug!("multi starting gameplay");
                self.queue_state_change(GameState::Ingame(Box::new(manager)));
            }
        }

        match packet {
            MultiplayerPacket::Server_LobbyList { lobbies } => {
                self.online_manager.lobbies = lobbies;
            }

            MultiplayerPacket::Server_LobbyInvite { 
                inviter_id, 
                lobby 
            } => {
                let username = if let Some(user) 
                    = self.online_manager.users.get(&inviter_id)
                {
                    user.username.clone()
                } else {
                    "A user".to_owned()
                };

                // mark the lobby we have saved as without password, so the user isnt prompted to enter the password
                // since we were invited, we can join without the password
                if let Some(l) = self.online_manager.lobby(lobby.id) { 
                    l.has_password = false;
                }

                self.actions.push(
                    Notification::default()
                    .text(format!("{username} has invited you to a multiplayer match"))
                    .color(Color::PURPLE_AMETHYST)
                    .duration(10_000.0)
                    .onclick(NotificationOnClick::MultiplayerLobby(lobby.id))
                    .into()
                );
            }

            MultiplayerPacket::Server_CreateLobby { 
                success, 
                lobby 
            } => {
                let Some(lobby) = lobby.filter(|_| success) 
                else { 
                    warn!("no success or lobby"); 
                    return Ok(()) 
                };

                if !self.online_manager.multiplayer_data.lobby_creation_pending { 
                    warn!("no join pending"); 
                    return Ok(()) 
                }

                let our_id = self.online_manager.user_id;
                if our_id == 0 { warn!("user_id == 0"); return Ok(()) }


                let mut info = CurrentLobbyInfo::new(
                    lobby, 
                    our_id
                );
                self.online_manager.update_usernames(&mut info);

                let manager = MultiplayerManager::new(
                    info, 
                    self.global.gamemode_infos.clone(),
                    &mut self.actions,
                );
                manager.update_values(&mut self.values);
                self.multiplayer_manager = Some(Box::new(manager));
                #[cfg(feature="graphics")]
                self.handle_custom_menu("lobby_menu");


                // try to update the server with our current map and mode
                let Some(map_hash) = self.values.current_beatmap_prop(
                    |b| b.beatmap_hash
                ) 
                else { return Ok(()) };

                let Some(map) = self.beatmap_manager
                    .get_by_hash(&map_hash) 
                else { return Ok(()) };

                let mode = self.global.playmode.clone();
                self.online_manager.update_lobby_beatmap(&map, mode.to_string());
            }
            MultiplayerPacket::Server_JoinLobby { 
                success, 
                lobby 
            } => {
                let Some(lobby) = lobby.filter(|_| success) 
                else { 
                    return Ok(()) 
                };
                if !self.online_manager.multiplayer_data.lobby_join_pending { 
                    return Ok(()) 
                }

                let our_id = self.online_manager.user_id;
                if our_id == 0 { return Ok (()) }

                let mut info = CurrentLobbyInfo::new(
                    lobby, 
                    our_id
                );
                self.online_manager.update_usernames(&mut info);

                let manager = MultiplayerManager::new(
                    info, 
                    self.global.gamemode_infos.clone(),
                    &mut self.actions,
                );
                manager.update_values(&mut self.values);
                self.multiplayer_manager = Some(Box::new(manager));
                #[cfg(feature="graphics")]
                self.handle_custom_menu("lobby_menu");
            }


            MultiplayerPacket::Server_LobbyCreated { lobby } => {
                self.online_manager.lobbies.push(lobby);
            }
            MultiplayerPacket::Server_LobbyDeleted { lobby_id } => {
                self.online_manager
                    .lobbies
                    .iter()
                    .enumerate()
                    .find(|(_, l)| l.id == lobby_id)
                    .map(|(i, _)| i)
                    .map(|index| self.online_manager.lobbies.remove(index));
            }

            MultiplayerPacket::Server_LobbyUserJoined { 
                lobby_id, 
                user_id 
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.players.push(user_id);
                }
            }

            MultiplayerPacket::Server_LobbyUserLeft { 
                lobby_id, 
                user_id 
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.players.retain(|u| u != &user_id);
                }

                if let Some(manager) = &self
                    .multiplayer_manager
                && manager.lobby.our_user_id == user_id {
                    self.multiplayer_manager = None;
                    self.actions.push(
                        Notification::default()
                        .text("You have been kicked from the match")
                        .duration(3000.0)
                        .color(Color::PURPLE)
                        .into()
                    );
                }
            }


            MultiplayerPacket::Server_LobbyMapChange { 
                lobby_id, 
                new_map
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.current_beatmap = Some(new_map.title.clone());
                };
            }

            MultiplayerPacket::Server_LobbyStateChange { 
                lobby_id, 
                new_state 
            } => {
                if let Some(l) = self.online_manager.lobby(lobby_id) { 
                    l.state = new_state;
                }
            }

            _ => {}
        }

        Ok(())
    }

}
