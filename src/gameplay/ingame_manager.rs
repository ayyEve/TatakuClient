use prelude::helpers::score_helper::ScoreLoaderHelper;

use crate::prelude::*;
use crate::beatmaps::osu::hitobject_defs::HitSamples;

/// how much time should pass at beatmap start before audio begins playing (and the map "starts")
const LEAD_IN_TIME:f32 = 1000.0;
/// how long should the offset be drawn for?
const OFFSET_DRAW_TIME:f32 = 2_000.0;
/// how tall is the duration bar
pub const DURATION_HEIGHT:f64 = 35.0;


/// ms between spectator score sync packets
const SPECTATOR_SCORE_SYNC_INTERVAL:f32 = 1000.0;


pub struct IngameManager {
    pub beatmap: Beatmap,
    pub metadata: BeatmapMeta,
    pub gamemode: Box<dyn GameMode>,
    pub current_mods: Arc<ModManager>,
    pub beatmap_preferences: BeatmapPreferences,

    pub score: IngameScore,
    pub replay: Replay,
    pub health: HealthHelper,

    pub key_counter: KeyCounter,

    ui_elements: Vec<UIElement>,

    pub score_list: Vec<IngameScore>,
    score_loader: Option<Arc<RwLock<ScoreLoaderHelper>>>,

    pub started: bool,
    pub completed: bool,
    pub replaying: bool,
    pub failed: bool,
    pub failed_time: f32,

    /// should the manager be paused?
    pub should_pause: bool,

    /// is a pause pending?
    /// used for breaks. if the user tabs out during a break, a pause is pending, but we shouldnt pause
    pause_pending: bool,

    /// is this playing in the background of the main menu?
    pub menu_background: bool,
    pub end_time: f32,

    pub lead_in_time: f32,
    pub lead_in_timer: Instant,

    pub timing_points: Vec<TimingPoint>,
    pub timing_point_index: usize,

    #[cfg(feature="bass_audio")]
    pub song: StreamChannel,
    #[cfg(feature="bass_audio")]
    pub hitsound_cache: HashMap<String, Option<SampleChannel>>,

    #[cfg(feature="neb_audio")] 
    pub song: Weak<AudioHandle>,
    #[cfg(feature="neb_audio")] 
    pub hitsound_cache: HashMap<String, Option<Sound>>,


    // offset things
    // TODO: merge these into one text helper, so they dont overlap
    offset: CenteredTextHelper<f32>,
    global_offset: CenteredTextHelper<f32>,


    /// (map.time, note.time - hit.time)
    pub hitbar_timings: Vec<(f32, f32)>,

    /// list of judgement indicators to draw
    judgement_indicators: Vec<Box<dyn JudgementIndicator>>,

    /// if in replay mode, what replay frame are we at?
    replay_frame: u64,

    pub background_game_settings: BackgroundGameSettings,
    pub common_game_settings: Arc<CommonGameplaySettings>,

    // spectator variables
    // TODO: should these be in their own struct? it might simplify things

    /// when was the last score sync packet sent?
    last_spectator_score_sync: f32,
    pub spectator_cache: Vec<(u32, String)>,

    /// what should the game do on start?
    /// mainly a helper for spectator
    pub on_start: Box<dyn FnOnce(&mut Self)>,

    pub events: Vec<InGameEvent>,

    ui_editor: Option<GameUIEditorDialog>
}

impl IngameManager {
    pub fn new(beatmap: Beatmap, gamemode: Box<dyn GameMode>) -> Self {
        let playmode = gamemode.playmode();
        let metadata = beatmap.get_beatmap_meta();

        let settings = get_settings!();
        let timing_points = beatmap.get_timing_points();
        let font = get_font();
        let hitsound_cache = HashMap::new();
        let current_mods = Arc::new(ModManager::get().clone());
        let common_game_settings = Arc::new(settings.common_game_settings.clone().init());

        let mut score =  Score::new(beatmap.hash().clone(), settings.username.clone(), playmode.clone());
        score.speed = current_mods.speed;

        let health = HealthHelper::new(Some(metadata.hp));
        let beatmap_preferences = Database::get_beatmap_prefs(&metadata.beatmap_hash);

        let score_loader = Some(SCORE_HELPER.read().get_scores(&metadata.beatmap_hash, &playmode));

        let key_counter = KeyCounter::new(gamemode.get_possible_keys().into_iter().map(|a| (a.0, a.1.to_owned())).collect());

        Self {
            metadata,
            timing_points,
            hitsound_cache,
            current_mods,
            health,
            key_counter,

            lead_in_timer: Instant::now(),
            score: IngameScore::new(score, true, false),

            replay: Replay::new(),
            beatmap,

            #[cfg(feature="bass_audio")]
            song: Audio::get_song().unwrap_or(create_empty_stream()), // temp until we get the audio file path
            #[cfg(feature="neb_audio")]
            song: Weak::new(),

            lead_in_time: LEAD_IN_TIME,
            end_time: gamemode.end_time(),

            offset: CenteredTextHelper::new("Offset", beatmap_preferences.audio_offset, OFFSET_DRAW_TIME, -20.0, font.clone()),
            global_offset: CenteredTextHelper::new("Global Offset", 0.0, OFFSET_DRAW_TIME, -20.0, font.clone()),
            beatmap_preferences,

            background_game_settings: settings.background_game_settings.clone(),
            common_game_settings,

            gamemode,
            score_list: Vec::new(),
            score_loader,
            // initialize defaults for anything else not specified
            ..Self::default()
        }
    }

    fn init_ui(&mut self) {
        if self.ui_editor.is_some() {return}

        let window_size = Settings::window_size();
        
        let playmode = self.gamemode.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };

        // score
        self.ui_elements.push(UIElement::new(
            &get_name("score"),
            Vector2::new(window_size.x, 0.0),
            ScoreElement::new()
        ));

        // combo
        self.ui_elements.push(UIElement::new(
            &get_name("combo"),
            Vector2::new(window_size.x, window_size.y),
            ComboElement::new(self.gamemode.combo_bounds())
        ));

        // Acc
        self.ui_elements.push(UIElement::new(
            &get_name("acc"),
            Vector2::new(window_size.x, 40.0),
            AccuracyElement::new()
        ));

        // Healthbar
        self.ui_elements.push(UIElement::new(
            &get_name("healthbar"),
            Vector2::zero(),
            HealthBarElement::new(self.common_game_settings.clone())
        ));

        // Duration Bar
        self.ui_elements.push(UIElement::new(
            &get_name("durationbar"),
            Vector2::new(0.0, window_size.y),
            DurationBarElement::new(self.common_game_settings.clone())
        ));

        // Judgement Bar
        self.ui_elements.push(UIElement::new(
            &get_name("judgementbar"),
            Vector2::new(0.0, window_size.y),
            JudgementBarElement::new(self.gamemode.timing_bar_things())
        ));

        // Key Counter
        self.ui_elements.push(UIElement::new(
            &get_name("key_counter"),
            Vector2::new(window_size.x, window_size.y/2.0),
            KeyCounterElement::new()
        ));

        // Spectators
        self.ui_elements.push(UIElement::new(
            &get_name("spectators"),
            Vector2::new(0.0, window_size.y/3.0),
            SpectatorsElement::new()
        ));

        // Leaderboard
        self.ui_elements.push(UIElement::new(
            &get_name("leaderboard"),
            self.gamemode.score_draw_start_pos(),
            LeaderboardElement::new()
        ));

        // judgement counter
        self.ui_elements.push(UIElement::new(
            &get_name("judgement_counter"),
            Vector2::new(window_size.x, window_size.y * (2.0/3.0)),
            JudgementCounterElement::new()
        ));
    }

    pub fn apply_mods(&mut self, mods: ModManager) {
        if self.started {
            NotificationManager::add_text_notification("Error applying mods to IngameManager\nmap already started", 2000.0, Color::RED);
        } else {
            self.current_mods = Arc::new(mods);
            // update replay speed too
            // TODO: add mods to replay data instead of this shit lmao
            self.replay.speed = self.current_mods.speed;
        }
    }

    // interactions with game mode

    pub fn play_note_sound(&mut self, note_time:f32, note_hitsound: u8, note_hitsamples:HitSamples) {
        let timing_point = self.beatmap.control_point_at(note_time);
        
        let mut play_normal = true; //(note_hitsound & 1) > 0; // 0: Normal
        let play_whistle = (note_hitsound & 2) > 0; // 1: Whistle
        let play_finish = (note_hitsound & 4) > 0; // 2: Finish
        let play_clap = (note_hitsound & 8) > 0; // 3: Clap

        // get volume
        let mut vol = (if note_hitsamples.volume == 0 {timing_point.volume} else {note_hitsamples.volume} as f32 / 100.0) * get_settings!().get_effect_vol();
        if self.menu_background {vol *= self.background_game_settings.hitsound_volume};


        // https://osu.ppy.sh/wiki/en/osu%21_File_Formats/Osu_%28file_format%29#hitsounds

        // normalSet and additionSet can be any of the following:
        // 0: No custom sample set
        // For normal sounds, the set is determined by the timing point's sample set.
        // For additions, the set is determined by the normal sound's sample set.
        // 1: Normal set
        // 2: Soft set
        // 3: Drum set

        // The filename is <sampleSet>-hit<hitSound><index>.wav, where:

        // sampleSet is normal, soft, or drum, determined by either normalSet or additionSet depending on which hitsound is playing
        const SAMPLE_SETS:&[&str] = &["normal", "normal", "soft", "drum"];
        // hitSound is normal, whistle, finish, or clap
        // index is the same index as above, except it is not written if the value is 0 or 1

        // (filename, index)
        let mut play_list = Vec::new();

        // if the hitsound is being overridden
        if let Some(name) = note_hitsamples.filename {
            if name.len() > 0 {
                #[cfg(feature="debug_hitsounds")]
                debug!("got custom sound: {}", name);
                if exists(format!("resources/audio/{}", name)) {
                    play_normal = (note_hitsound & 1) > 0;
                    play_list.push((name, 0))
                } else {
                    #[cfg(feature="debug_hitsounds")]
                    warn!("doesnt exist");
                }
            }
        }

        if play_normal {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitnormal.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }

        if play_whistle {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitwhistle.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }
        if play_finish {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitfinish.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }
        if play_clap {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitclap.wav", sample_set);
            let index = note_hitsamples.index;
            // if sample_set == 0 {sample_set = timing_point.sample_set}
            // if index == 1 {} //idk wtf 

            play_list.push((hitsound, index))
        }


        // The sound file is loaded from the first of the following directories that contains a matching filename:
        // Beatmap, if index is not 0
        // Skin, with the index removed
        // Default osu! resources, with the index removed
        // When filename is given, no addition sounds will be played, and this file in the beatmap directory is played instead.

        // debug!("{}, {} | {}", timing_point.volume, note_hitsamples.volume, );


        for (sound_file, _index) in play_list.iter() {
            if !self.hitsound_cache.contains_key(sound_file) {
                #[cfg(feature="debug_hitsounds")]
                trace!("not cached");

                #[cfg(feature="bass_audio")]
                let sound = Audio::load(format!("resources/audio/{}", sound_file));
                #[cfg(feature="neb_audio")]
                let sound = crate::game::Sound::load(format!("resources/audio/{}", sound_file));

                if let Err(e) = &sound {
                    error!("error loading: {:?}", e);
                }
                
                self.hitsound_cache.insert(sound_file.clone(), sound.ok());
            }

            if let Some(sound) = self.hitsound_cache.get(sound_file).unwrap() {
                #[cfg(feature="bass_audio")] {
                    sound.set_volume(vol).unwrap();
                    sound.set_position(0.0).unwrap();
                    sound.play(true).unwrap();
                }
                #[cfg(feature="neb_audio")] {
                    let sound = Audio::play_sound(sound.clone());
                    if let Some(sound) = sound.upgrade() {
                        sound.set_volume(vol);
                        sound.set_position(0.0);
                        sound.play();
                    }
                }
            }
        }
    }

    pub fn add_judgement_indicator<HI:JudgementIndicator+'static>(&mut self, mut indicator: HI) {
        indicator.set_draw_duration(self.common_game_settings.hit_indicator_draw_duration);
        self.judgement_indicators.push(Box::new(indicator))
    }


    pub fn update(&mut self) {
        // update ui elements
        let mut ui_elements = std::mem::take(&mut self.ui_elements);
        ui_elements.iter_mut().for_each(|ui|ui.update(self));
        self.ui_elements = ui_elements;

        
        let mut ui_editor = std::mem::take(&mut self.ui_editor);
        let mut should_close = false;
        if let Some(ui_editor) = &mut ui_editor {
            ui_editor.update(&mut ());
            ui_editor.update_elements(self);

            if ui_editor.should_close() {
                self.ui_elements = std::mem::take(&mut ui_editor.elements);
                should_close = true
            }
        }
        if !should_close {
            self.ui_editor = ui_editor;
        }
        

        // check lead-in time
        if self.lead_in_time > 0.0 {
            let elapsed = self.lead_in_timer.elapsed().as_micros() as f32 / 1000.0;
            self.lead_in_timer = Instant::now();
            self.lead_in_time -= elapsed * self.game_speed();

            if self.lead_in_time <= 0.0 {

                #[cfg(feature="bass_audio")] {
                    self.song.set_position(-self.lead_in_time as f64).unwrap();
                    self.song.set_volume(get_settings!().get_music_vol()).unwrap();
                    self.song.set_rate(self.game_speed()).unwrap();
                    self.song.play(true).unwrap();
                }
                
                #[cfg(feature="neb_audio")] {
                    let song = self.song.upgrade().unwrap();
                    song.set_position(-self.lead_in_time);
                    song.set_volume(get_settings!().get_music_vol());
                    song.set_playback_speed(self.game_speed() as f64);
                    song.play();
                }

                self.lead_in_time = 0.0;
            }
        }
        let time = self.time();

        // check timing point
        let timing_points = &self.timing_points;
        if self.timing_point_index + 1 < timing_points.len() && timing_points[self.timing_point_index + 1].time <= time {
            self.timing_point_index += 1;
        }

        // check if scores have been loaded
        if let Some(loader) = self.score_loader.clone() {
            let loader = loader.read();
            if loader.done {
                self.score_list = loader.scores.iter().map(|s|IngameScore::new(s.clone(), false, s.username == self.score.username)).collect();
                self.score_loader = None;
            }
        }

        let mut gamemode = std::mem::take(&mut self.gamemode);

        // read inputs from replay if replaying
        if self.replaying && !self.current_mods.autoplay {

            // read any frames that need to be read
            loop {
                if self.replay_frame as usize >= self.replay.frames.len() {break}
                
                let (frame_time, frame) = self.replay.frames[self.replay_frame as usize];
                if frame_time > time {break}

                gamemode.handle_replay_frame(frame, frame_time, self);
                
                self.replay_frame += 1;
            }
        }

        // update hit timings bar
        self.hitbar_timings.retain(|(hit_time, _)| {time - hit_time < HIT_TIMING_DURATION});
        
        // update judgement indicators
        self.judgement_indicators.retain(|a| a.should_keep(time));

        // update gamemode
        gamemode.update(self, time);

        if self.song.get_playback_state().unwrap() == PlaybackState::Stopped {
            trace!("Song over, saying map is complete");
            self.completed = true;
        }


        // do fail things
        // TODO: handle edge cases, like replays, spec, autoplay, etc
        if self.failed {
            let new_rate = f64::lerp(self.game_speed() as f64, 0.0, (self.time() - self.failed_time) as f64 / 1000.0) as f32;

            if new_rate <= 0.05 {
                #[cfg(feature="bass_audio")]
                self.song.pause().unwrap();
            
                #[cfg(feature="neb_audio")]
                if let Some(song) = self.song.upgrade() {
                    song.pause()
                }

                self.completed = true;
                // self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorFrameData::Failed));
                trace!("show fail menu");
            } else {
                #[cfg(feature="bass_audio")]
                self.song.set_rate(new_rate).unwrap();

                #[cfg(feature="neb_audio")]
                if let Some(song) = self.song.upgrade() {
                    song.set_playback_speed(new_rate as f64)
                }
            }

            // put it back
            self.gamemode = gamemode;
            return;
        }

        // send map completed packets
        if self.completed {
            self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorFrameData::ScoreSync {score: self.score.score.clone()}));
            self.outgoing_spectator_frame_force((self.end_time + 10.0, SpectatorFrameData::Buffer));
        }

        // update our spectator list if we can
        if let Ok(manager) = ONLINE_MANAGER.try_read() {
            self.spectator_cache = manager.spectator_list.clone()
        }

        // if its time to send another score sync packet
        if self.last_spectator_score_sync + SPECTATOR_SCORE_SYNC_INTERVAL <= time {
            self.last_spectator_score_sync = time;
            
            // create and send the packet
            self.outgoing_spectator_frame((time, SpectatorFrameData::ScoreSync {score: self.score.score.clone()}))
        }

        // put it back
        self.gamemode = gamemode;
    }

    pub fn draw(&mut self, args: RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        let time = self.time();

        // draw gamemode
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.draw(args, self, list);
        self.gamemode = gamemode;

        
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.draw(&args, &0.0, list);
        } 


        // draw center texts
        self.offset.draw(time, list);
        self.global_offset.draw(time, list);


        // dont draw score, combo, etc if this is a menu bg
        if self.menu_background {return}


        // gamemode things

        // draw ui elements
        for i in self.ui_elements.iter_mut() {
            i.draw(list)
        }

        
        // draw judgement indicators
        for indicator in self.judgement_indicators.iter_mut() {
            indicator.draw(time, list);
        }

    }
}

// getters, setters, properties
impl IngameManager {
    pub fn all_scores(&self) -> Vec<&IngameScore> {
        let mut list = Vec::new();
        for score in self.score_list.iter() {
            list.push(score)
        }

        list.push(&self.score);

        // sort by points
        list.sort_by(|a,b| b.score.score.cmp(&a.score.score));

        list
    }

    pub fn time(&mut self) -> f32 {
        #[cfg(feature="bass_audio")]
        let t = self.song.get_position().unwrap() as f32;

        #[cfg(feature="neb_audio")]
        let t = match (self.song.upgrade(), Audio::get_song_raw()) {
            (None, Some((_, song))) => {
                match song.upgrade() {
                    Some(s) => {
                        self.song = song;
                        s.current_time()
                    }
                    None => {
                        warn!("song doesnt exist at Beatmap.time()!!");
                        self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
                        self.song.upgrade().unwrap().pause();
                        0.0
                    }
                }
            },
            (None, None) => {
                warn!("song doesnt exist at Beatmap.time()!!");
                self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
                self.song.upgrade().unwrap().pause();
                0.0
            }
            (Some(song), _) => song.current_time(),
        };

        t - (self.lead_in_time + self.offset.value + self.global_offset.value)
    }

    pub fn should_save_score(&self) -> bool {
        let should = !(self.replaying || self.current_mods.autoplay);
        should
    }

    // is this game pausable
    pub fn can_pause(&mut self) -> bool {
        self.should_pause || !(self.current_mods.autoplay || self.replaying || self.failed)
    }

    #[inline]
    pub fn game_speed(&self) -> f32 {
        if self.menu_background {
            1.0 // TODO: 
        } else if self.replaying {
            // if we're replaying, make sure we're using the score's speed
            self.replay.speed
        } else {
            self.current_mods.speed
        }
    }


    pub fn current_timing_point(&self) -> TimingPoint {
        self.timing_points[self.timing_point_index]
    }
    pub fn timing_point_at(&self, time: f32, allow_inherited: bool) -> &TimingPoint {
        let mut tp = &self.timing_points[0];

        for i in self.timing_points.iter() {
            if i.is_inherited() && !allow_inherited {continue}
            if i.time <= time {
                tp = i
            }
        }

        tp
    }


}

// Events and States
impl IngameManager {

    // can be from either paused or new
    pub fn start(&mut self) {
        if !self.gamemode.show_cursor() {
            if !self.menu_background {
                CursorManager::set_visible(false)
            }
        } else if self.replaying || self.current_mods.autoplay {
            CursorManager::show_system_cursor(true)
        }

        self.pause_pending = false;
        self.should_pause = false;

        // re init ui because pointers may not be valid anymore
        self.ui_elements.clear();
        self.init_ui();

        if !self.started {
            self.reset();

            if !self.replaying {
                self.outgoing_spectator_frame((0.0, SpectatorFrameData::Play {
                    beatmap_hash: self.beatmap.hash(),
                    mode: self.gamemode.playmode(),
                    mods: serde_json::to_string(&(*self.current_mods)).unwrap()
                }));
            }

            if self.menu_background {
                // dont reset the song, and dont do lead in
                self.lead_in_time = 0.0;
            } else {
                #[cfg(feature="bass_audio")] {
                    self.song.set_position(0.0).unwrap();
                    self.song.pause().unwrap();
                    if self.replaying {
                        self.song.set_rate(self.replay.speed).unwrap();
                    }
                }

                #[cfg(feature="neb_audio")]
                match self.song.upgrade() {
                    Some(song) => {
                        song.set_position(0.0);
                        if self.replaying {
                            song.set_playback_speed(self.replay.speed as f64)
                        }
                    }
                    None => {
                        self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
                        self.song.upgrade().unwrap().pause();
                    }
                }
                
                self.lead_in_timer = Instant::now();
                self.lead_in_time = LEAD_IN_TIME;
            }

            // volume is set when the song is actually started (when lead_in_time is <= 0)
            self.started = true;

            // run the startup code
            let mut on_start:Box<dyn FnOnce(&mut Self)> = Box::new(|_|{});
            std::mem::swap(&mut self.on_start, &mut on_start);
            on_start(self);

        } else if self.lead_in_time <= 0.0 {
            // if this is the menu, dont do anything
            if self.menu_background {return}
            
            let frame = SpectatorFrameData::UnPause;
            let time = self.time();
            self.outgoing_spectator_frame((time, frame));

            #[cfg(feature="bass_audio")]
            self.song.play(false).unwrap();

            // // needed because if paused for a while it can crash
            #[cfg(feature="neb_audio")]
            match self.song.upgrade() {
                Some(song) => song.play(),
                None => self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0),
            }
        }
    }
    pub fn pause(&mut self) {

        // make sure the cursor is visible
        CursorManager::set_visible(true);
        CursorManager::show_system_cursor(false);

        #[cfg(feature="bass_audio")]
        let _ = self.song.pause();
        #[cfg(feature="neb_audio")]
        self.song.upgrade().unwrap().pause();

        // is there anything else we need to do?

        // might mess with lead-in but meh

        let time = self.time();
        self.outgoing_spectator_frame_force((time, SpectatorFrameData::Pause));
    }
    pub fn reset(&mut self) {
        let settings = get_settings!();
        
        self.gamemode.reset(&self.beatmap);
        self.health.reset();
        self.key_counter.reset();
        self.hitbar_timings.clear();
        self.judgement_indicators.clear();

        if self.menu_background {
            self.background_game_settings = settings.background_game_settings.clone();
            self.gamemode.apply_auto(&self.background_game_settings)
        } else {
            // reset song
            #[cfg(feature="bass_audio")] {
                self.song.set_rate(self.game_speed()).unwrap();
                self.song.set_position(0.0).unwrap();
                let _ = self.song.pause();
            }
            
            #[cfg(feature="neb_audio")] 
            match self.song.upgrade() {
                Some(song) => {
                    song.set_position(0.0);
                    song.pause();
                    song.set_playback_speed(self.game_speed() as f64);
                }
                None => {
                    while let None = self.song.upgrade() {
                        self.song = Audio::play_song(self.metadata.audio_filename.clone(), true, 0.0);
                    }
                    let song = self.song.upgrade().unwrap();
                    song.set_playback_speed(self.game_speed() as f64);
                    song.pause();
                }
            }
        }

        self.completed = false;
        self.started = false;
        self.failed = false;
        self.lead_in_time = LEAD_IN_TIME;
        self.lead_in_timer = Instant::now();
        self.score = IngameScore::new(Score::new(self.beatmap.hash(), settings.username.clone(), self.gamemode.playmode()), true, false);
        self.replay_frame = 0;
        self.timing_point_index = 0;
        
        if !self.replaying {
            // only reset the replay if we arent replaying
            self.replay = Replay::new();
            self.score.speed = self.current_mods.speed;
        }
    }
    pub fn fail(&mut self) {
        if self.failed || self.current_mods.nofail || self.current_mods.autoplay || self.menu_background {return}
        self.failed = true;
        self.failed_time = self.time();
    }

    pub fn combo_break(&mut self) {
        // reset combo to 0
        self.score.combo = 0;
        
        // play hitsound
        if self.score.combo >= 20 && !self.menu_background {
            #[cfg(feature="bass_audio")]
            Audio::play_preloaded("combobreak").unwrap();
            #[cfg(feature="neb_audio")]
            Audio::play_preloaded("combobreak");
        }
    }

    // TODO: implement this properly, gamemode will probably have to handle some things too
    pub fn jump_to_time(&mut self, time: f32, skip_intro: bool) {
        #[cfg(feature="bass_audio")]
        self.song.set_position(time as f64).unwrap();

        #[cfg(feature="neb_audio")]
        if let Some(song) = self.song.upgrade() {
            song.set_position(time)
        }

        if skip_intro {
            self.lead_in_time = 0.0;
        }
    }

    pub fn on_complete(&mut self) {
        // make sure the cursor is visible
        CursorManager::set_visible(true);
        CursorManager::show_system_cursor(false);

    }
    
}

// Input Handlers
impl IngameManager {
    pub fn key_down(&mut self, key:piston::Key, mods: ayyeve_piston_ui::menu::KeyModifiers) {
        if (self.replaying || self.current_mods.autoplay) && !self.menu_background {
            // check replay-only keys
            if key == piston::Key::Escape {
                self.started = false;
                self.completed = true;
                return;
            }
        }

        if self.failed && key == piston::Key::Escape {
            // set the failed time to negative, so it triggers the end
            self.failed_time = -1000.0;
        }
        if self.failed {return}
        

        if key == Key::Escape && self.can_pause() {
            self.should_pause = true;
        }

        
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_key_press(&key, &mods, &mut ());
            if key == Key::F9 {
                ui_editor.should_close = true;
            }
        } else if key == Key::F9 {
            self.ui_editor = Some(GameUIEditorDialog::new(std::mem::take(&mut self.ui_elements)));
        }

        let mut gamemode = std::mem::take(&mut self.gamemode);

        // skip intro
        if key == piston::Key::Space {
            gamemode.skip_intro(self);
        }

        // check for offset changing keys
        {
            if mods.shift {
                let mut t = 0.0;
                if key == self.common_game_settings.key_offset_up {t = 5.0}
                if key == self.common_game_settings.key_offset_down {t = -5.0}

                if t != 0.0 {
                    self.increment_global_offset(t);
                }
            } else {
                if key == self.common_game_settings.key_offset_up {self.increment_offset(5.0)}
                if key == self.common_game_settings.key_offset_down {self.increment_offset(-5.0)}
            }
        }


        gamemode.key_down(key, self);
        self.gamemode = gamemode;
    }
    pub fn key_up(&mut self, key:piston::Key) {
        if self.failed {return}

        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.key_up(key, self);
        self.gamemode = gamemode;
    }
    pub fn on_text(&mut self, text:&String, mods: &ayyeve_piston_ui::menu::KeyModifiers) {
        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.on_text(text, mods, self);
        self.gamemode = gamemode;
    }
    
    
    pub fn mouse_move(&mut self, pos:Vector2) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_move(&pos, &mut ());
        }

        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_move(pos, self);
        self.gamemode = gamemode;
    }
    pub fn mouse_down(&mut self, btn:piston::MouseButton) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_down(&Vector2::zero(), &btn, &KeyModifiers::default(), &mut ());
            return
        }

        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_down(btn, self);
        self.gamemode = gamemode;
    }
    pub fn mouse_up(&mut self, btn:piston::MouseButton) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_up(&Vector2::zero(), &btn, &KeyModifiers::default(), &mut ());
            return
        }

        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_up(btn, self);
        self.gamemode = gamemode;
    }
    pub fn mouse_scroll(&mut self, delta:f64) {
        if let Some(ui_editor) = &mut self.ui_editor {
            ui_editor.on_mouse_scroll(&delta, &mut ());
        } 

        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.mouse_scroll(delta, self);
        self.gamemode = gamemode;
    }


    pub fn controller_press(&mut self, c: &Box<dyn Controller>, btn: u8) {
        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.controller_press(c, btn, self);
        self.gamemode = gamemode;
    }
    pub fn controller_release(&mut self, c: &Box<dyn Controller>, btn: u8) {
        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.controller_release(c, btn, self);
        self.gamemode = gamemode;
    }
    pub fn controller_axis(&mut self, c: &Box<dyn Controller>, axis_data:HashMap<u8, (bool, f64)>) {
        if self.failed {return}
        let mut gamemode = std::mem::take(&mut self.gamemode);
        gamemode.controller_axis(c, axis_data, self);
        self.gamemode = gamemode;
    }

    pub fn window_focus_lost(&mut self, got_focus: bool) {
        if got_focus {
            self.pause_pending = false
        } else {
            if self.can_pause() {
                // if self.in_break() {self.pause_pending = true} else {self.should_pause = true}
            }
        }
    }
}

// other misc stuff that isnt touched often and i just wanted it out of the way
impl IngameManager {
    
    pub fn increment_offset(&mut self, delta:f32) {
        let time = self.time();
        let new_val = self.offset.value + delta;
        self.offset.set_value(new_val, time);

        // update the beatmap offset
        self.beatmap_preferences.audio_offset = new_val;
        Database::save_beatmap_prefs(&self.beatmap.hash(), &self.beatmap_preferences);
    }
    /// locks settings
    pub fn increment_global_offset(&mut self, delta:f32) {
        let mut settings = get_settings_mut!();
        settings.global_offset += delta;

        let time = self.time();
        self.global_offset.set_value(settings.global_offset, time);
    }

}

// Spectator Stuff
impl IngameManager {
    pub fn outgoing_spectator_frame(&mut self, frame: SpectatorFrame) {
        if self.menu_background || self.replaying {return}
        OnlineManager::send_spec_frames(vec![frame], false)
    }
    pub fn outgoing_spectator_frame_force(&mut self, frame: SpectatorFrame) {
        if self.menu_background || self.replaying {return}
        OnlineManager::send_spec_frames(vec![frame], true);
    }

}

// default
impl Default for IngameManager {
    fn default() -> Self {
        Self { 
            #[cfg(feature="bass_audio")]
            song: create_empty_stream(),
            #[cfg(feature="neb_audio")]
            song: Weak::new(),
            judgement_indicators: Vec::new(),

            failed: false,
            failed_time: 0.0,
            health: HealthHelper::default(),
            beatmap: Default::default(),
            metadata: Default::default(),
            gamemode: Default::default(),
            current_mods: Default::default(),
            score: IngameScore::new(Default::default(), true, false),
            replay: Default::default(),
            started: Default::default(),
            completed: Default::default(),
            replaying: Default::default(),
            menu_background: Default::default(),
            end_time: Default::default(),
            lead_in_time: Default::default(),
            lead_in_timer: Instant::now(),
            timing_points: Default::default(),
            timing_point_index: Default::default(),
            hitsound_cache: Default::default(),
            offset: Default::default(),
            global_offset: Default::default(),
            hitbar_timings: Default::default(),
            replay_frame: Default::default(),
            background_game_settings: Default::default(), 
            spectator_cache: Default::default(),
            last_spectator_score_sync: 0.0,
            on_start: Box::new(|_|{}),

            common_game_settings: Default::default(),

            score_list: Vec::new(),
            score_loader: None,
            beatmap_preferences: Default::default(),
            should_pause: false,
            pause_pending: false,
            events: Vec::new(),
            ui_elements: Vec::new(),
            ui_editor: None,
            key_counter: KeyCounter::default()
        }
    }
}



pub trait GameMode {
    fn new(beatmap:&Beatmap, diff_calc_only: bool) -> Result<Self, TatakuError> where Self: Sized;
    fn score_hit_string(_hit:&ScoreHit) -> String where Self: Sized;
    fn show_cursor(&self) -> bool {false}
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)>;
    
    fn playmode(&self) -> PlayMode;

    fn end_time(&self) -> f32;
    fn combo_bounds(&self) -> Rectangle;
    fn score_draw_start_pos(&self) -> Vector2 {Vector2::new(0.0, 200.0)}
    /// f64 is hitwindow, color is color for that window. last is miss hitwindow
    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color));
    /// convert mouse pos to mode's playfield coords
    // fn scale_mouse_pos(&self, mouse_pos:Vector2) -> Vector2 {mouse_pos}

    fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager);

    fn update(&mut self, manager:&mut IngameManager, time: f32);
    fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut Vec<Box<dyn Renderable>>);

    fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager);
    fn on_text(&mut self, _text: &String, _mods: &KeyModifiers, _manager: &mut IngameManager) {}

    fn mouse_move(&mut self, _pos:Vector2, _manager:&mut IngameManager) {}
    fn mouse_down(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    fn mouse_up(&mut self, _btn:piston::MouseButton, _manager:&mut IngameManager) {}
    fn mouse_scroll(&mut self, _delta:f64, _manager:&mut IngameManager) {}

    fn apply_auto(&mut self, settings: &BackgroundGameSettings);


    fn controller_press(&mut self, _c: &Box<dyn Controller>, _btn: u8, _manager:&mut IngameManager) {}
    fn controller_release(&mut self, _c: &Box<dyn Controller>, _btn: u8, _manager:&mut IngameManager) {}
    fn controller_hat_press(&mut self, _hat: piston::controller::ControllerHat, _manager:&mut IngameManager) {}
    fn controller_hat_release(&mut self, _hat: piston::controller::ControllerHat, _manager:&mut IngameManager) {}
    fn controller_axis(&mut self, _c: &Box<dyn Controller>, _axis_data:HashMap<u8, (bool, f64)>, _manager:&mut IngameManager) {}


    fn skip_intro(&mut self, manager: &mut IngameManager);
    fn pause(&mut self, _manager:&mut IngameManager) {}
    fn unpause(&mut self, _manager:&mut IngameManager) {}
    fn reset(&mut self, beatmap:&Beatmap);


}
impl Default for Box<dyn GameMode> {
    fn default() -> Self {
        Box::new(NoMode::new(&Default::default(), true).unwrap())
    }
}

// needed for std::mem::take/swap
struct NoMode {}
impl GameMode for NoMode {
    fn new(_:&Beatmap, _:bool) -> Result<Self, TatakuError> where Self: Sized {Ok(Self {})}
    fn playmode(&self) -> PlayMode {"osu".to_owned()}
    fn end_time(&self) -> f32 {0.0}
    fn combo_bounds(&self) -> Rectangle {Rectangle::bounds_only(Vector2::zero(), Vector2::zero())}
    fn timing_bar_things(&self) -> (Vec<(f32,Color)>, (f32,Color)) {(Vec::new(), (0.0, Color::WHITE))}
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {Vec::new()}

    fn handle_replay_frame(&mut self, _:ReplayFrame, _:f32, _:&mut IngameManager) {}
    fn update(&mut self, _:&mut IngameManager, _: f32) {}
    fn draw(&mut self, _:RenderArgs, _:&mut IngameManager, _: &mut Vec<Box<dyn Renderable>>) {}
    fn key_down(&mut self, _:piston::Key, _:&mut IngameManager) {}
    fn key_up(&mut self, _:piston::Key, _:&mut IngameManager) {}
    fn apply_auto(&mut self, _: &BackgroundGameSettings) {}
    fn skip_intro(&mut self, _: &mut IngameManager) {}
    fn reset(&mut self, _:&Beatmap) {}

    fn score_hit_string(_hit:&ScoreHit) -> String where Self: Sized {String::new()}
}

// struct HitsoundManager {
//     sounds: HashMap<String, Option<Sound>>,
//     /// sound, index
//     beatmap_sounds: HashMap<String, HashMap<u8, Sound>>
// }
// impl HitsoundManager {
// }

#[cfg(feature="bass_audio")]
lazy_static::lazy_static! {
    static ref EMPTY_STREAM:StreamChannel = {
        // wave file bytes with ~1 sample
        StreamChannel::create_from_memory(vec![0x52,0x49,0x46,0x46,0x28,0x00,0x00,0x00,0x57,0x41,0x56,0x45,0x66,0x6D,0x74,0x20,0x10,0x00,0x00,0x00,0x01,0x00,0x02,0x00,0x44,0xAC,0x00,0x00,0x88,0x58,0x01,0x00,0x02,0x00,0x08,0x00,0x64,0x61,0x74,0x61,0x04,0x00,0x00,0x00,0x80,0x80,0x80,0x80], 0i32).expect("error creating empty StreamChannel")
    };
}
#[cfg(feature="bass_audio")]
fn create_empty_stream() -> StreamChannel {
    EMPTY_STREAM.clone()
}



pub enum InGameEvent {
    Break {start: f32, end: f32}
}
