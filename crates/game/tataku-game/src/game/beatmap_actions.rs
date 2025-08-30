use crate::prelude::*;

// beatmap related actions 
impl Game {
    pub(super) fn set_current_beatmap(
        &mut self, 
        hash: Md5Hash,
        config: SelectBeatmapConfig
    ) {
        let beatmap = self.beatmap_manager.get_by_hash(&hash).unwrap();
        debug!(
            "Setting current beatmap to {} ({}) and playmode {}", 
            beatmap.beatmap_hash, 
            beatmap.file_path, 
            config.playmode
        );

        self.beatmap_manager.add_played(hash);

        // update value collection
        {
            let infos = &self.values.values.global.gamemode_infos;
            let actual_mode = infos.get_playmode_actual(
                &config.playmode, 
                Some(&beatmap)
            );

            // // let mods = &values.mods;
            // let diff = self.difficulty_manager.get_diff(
            //     beatmap, 
            //     actual_mode, 
            //     &config.mods
            // ).ok();


            // let diff_info = if let Ok(info) = infos.get_info(actual_mode) {
            //     let diff_meta = BeatmapWithDiff {
            //         map: beatmap.clone(),
            //         diff_rating: diff.unwrap_or(-1.0),
            //         diff_info: Box::default(),
            //     };

            //     info.diff_values
            //         .iter()
            //         .map(|dv| dv.format((dv.get_diff_value)(&diff_meta, &config.mods)))
            //         .collect::<Vec<_>>()
            //         .join(" | ")
            // } else {
            //     String::new()
            // };

            self.beatmap_manager.set_current(hash);
            
            self.actions.push(GameAction::UpdatePlaymodeActual(actual_mode.into()));

            // update beatmap settings provider
            let beatmap_prefs = Database::get_beatmap_prefs(hash);
            let playmode_prefs = Database::get_beatmap_mode_prefs(
                hash, 
                actual_mode
            );
            
            self.values.values.beatmap_settings = BeatmapSettings::new(
                beatmap_prefs, 
                playmode_prefs, 
                &mut self.values, 
                "beatmap_settings"
            );
        }

        // set the song
        let position = if config.use_preview_time { beatmap.audio_preview } 
            else { 0.0 };

        self.actions.push(SongAction::Set(SongSetAction::FromFile(
            beatmap.audio_filename.clone(), 
            SongPlayData {
                play: true,
                restart: config.restart_song,
                position: Some(position),
                ..Default::default()
            }
        )));
        // make sure the song is playing
        self.actions.push(SongAction::Play);
        // make sure to update the background
        self.actions.push(GameAction::UpdateBackground);

    }
    
    pub(super) fn remove_current_beatmap(&mut self) {
        trace!("Setting current beatmap to None");
        self.beatmap_manager.current_beatmap = None;

        // stop song
        self.actions.push(SongAction::Stop);
        self.actions.push(GameAction::UpdateBackground);
    }
    
    pub(super) fn delete_beatmap(
        &mut self, 
        beatmap: Md5Hash, 
        post_delete: PostDelete,
        config: SelectBeatmapConfig, 
    ) {
        if self.beatmap_manager.delete_beatmap(beatmap) {
            match post_delete {
                // select next beatmap
                PostDelete::Next => { 
                    self.next_beatmap(config); 
                }

                PostDelete::Previous => { 
                    self.previous_beatmap(config); 
                }

                PostDelete::Random => {
                    let Some(beatmap) = self.beatmap_manager
                        .random_beatmap() 
                    else { return };

                    self.set_current_beatmap(beatmap, config);
                }
            }
        }
    }

    pub(super) fn next_beatmap(
        &mut self, 
        config: SelectBeatmapConfig, 
    ) -> bool {
        match self.beatmap_manager.next_beatmap() {
            Some(map) => {
                self.set_current_beatmap(map, config);
                // since we're playing something already in the queue, dont append it again
                self.beatmap_manager.remove_played(0);
                true
            }

            None => if let Some(map) = self.beatmap_manager.random_beatmap() {
                self.set_current_beatmap(map, config);
                true
            } else {
                false
            }
        }
    }

    pub(super) fn previous_beatmap(
        &mut self, 
        config: SelectBeatmapConfig
    ) -> bool {
        match self.beatmap_manager.previous_beatmap() {
            Some(map) => {
                self.set_current_beatmap(map, config);
                // since we're playing something already in the queue, dont append it again
                // and undo the index bump done in set_current_beatmap
                self.beatmap_manager.remove_played(2);
                true
            }
            None => false
        }
    }
}