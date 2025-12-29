use crate::prelude::*;

use common::Md5Hash;
use engine::actions;
use actions::beatmap::PostDelete;

// beatmap related actions 
impl Game {
    pub(super) fn set_current_beatmap(
        &mut self, 
        hash: Md5Hash,
        config: &SelectBeatmapConfig,
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
            
            self.actions.push(actions::game::GameAction::UpdatePlaymodeActual(
                actual_mode.into()
            ).into());

            // update beatmap settings provider
            let beatmap_prefs = self
                .database
                .get_beatmap_preferences(hash)
                .unwrap_or_default();
            let playmode_prefs = self
                .database
                .get_beatmap_playmode_preferences(
                    hash, 
                    actual_mode
                )
                .unwrap_or_default();
            
            self.values.values.beatmap_settings = BeatmapSettings::new(
                beatmap_prefs, 
                playmode_prefs, 
                &mut self.values, 
                "beatmap_settings"
            );
        }

        // set the song
        let position = if config.use_preview_time { beatmap.audio_preview } else { 0.0 };

        self.actions.push(actions::song::SongAction::Set(actions::song::SongSetAction::FromFile(
            beatmap.audio_filename.clone(), 
            actions::song::SongPlayData {
                play: true,
                restart: config.restart_song,
                position: Some(position),
                ..Default::default()
            }
        )).into());
        // make sure the song is playing
        self.actions.push(actions::song::SongAction::Play.into());
        // make sure to update the background
        self.actions.push(actions::game::GameAction::UpdateBackground.into());
    }
    
    pub(super) fn remove_current_beatmap(&mut self) {
        trace!("Setting current beatmap to None");
        self.beatmap_manager.current_beatmap = None;

        // stop song
        self.actions.push(actions::song::SongAction::Stop.into());
        self.actions.push(actions::game::GameAction::UpdateBackground.into());
    }
    
    pub(super) fn delete_beatmap(
        &mut self, 
        beatmap: Md5Hash, 
        post_delete: PostDelete,
        config: &SelectBeatmapConfig, 
    ) {
        if self.values.beatmap_manager.delete_beatmap(beatmap) {
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
        config: &SelectBeatmapConfig, 
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
        config: &SelectBeatmapConfig,
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
