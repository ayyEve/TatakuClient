/**
 * Mania game mode
 * Authored by ayyEve
 * scroll velocity by Nebula
 * 
 * 
 * depth doc:
 * tbi
 */

use crate::prelude::*;
use super::prelude::*;

const FIELD_DEPTH:f64 = 110.0;
const HIT_AREA_DEPTH: f64 = 99.9;

pub const MANIA_NOTE_DEPTH: f64 = 100.0;


pub struct ManiaGame {
    map_meta: Arc<BeatmapMeta>,
    // lists
    pub columns: Vec<Vec<Box<dyn ManiaHitObject>>>,
    timing_bars: Vec<TimingBar>,

    position_function: Arc<Vec<PositionPoint>>,

    // list indices
    column_indices: Vec<usize>,
    /// true if held
    column_states: Vec<bool>,

    end_time: f32,
    sv_mult: f64,
    column_count: u8,

    auto_helper: ManiaAutoHelper,
    playfield: Arc<ManiaPlayfield>,

    game_settings: Arc<ManiaSettings>,

    mania_skin_settings: Option<Arc<ManiaSkinSettings>>,
    map_preferences: BeatmapPlaymodePreferences,

    key_images_up: HashMap<u8, Image>,
    key_images_down: HashMap<u8, Image>,

    hit_windows: Vec<(ManiaHitJudgments, Range<f32>)>,
    miss_window: f32,
}
impl ManiaGame {
    pub fn get_color(&self, col:u8) -> Color {
        match col {
            0|3 => Color::BLUE_ORCHID,
            1|2 => Color::ACID_GREEN,

            _ => Color::WHITE
        }
    }

    fn next_note(&mut self, col:usize) {
        (*self.column_indices.get_mut(col).unwrap()) += 1;
    }

    fn integrate_velocity(&mut self, mut slider_velocities: Vec<SliderVelocity>) {
        let mut position_function = vec![PositionPoint::default()];

        if slider_velocities.is_empty() {
            position_function.push(PositionPoint { 
                time: self.end_time,
                position: self.end_time as f64,
            });

            self.position_function = Arc::new(position_function);

            for col in self.columns.iter_mut() {
                for note in col.iter_mut() {
                    note.set_position_function(self.position_function.clone());
                }
            }

            for tl in self.timing_bars.iter_mut() {
                tl.set_position_function(self.position_function.clone());
            }

            return;
        }

        slider_velocities.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        let final_sv = SliderVelocity {
            time: self.end_time,
            slider_velocity: slider_velocities.last().unwrap().slider_velocity,
        };

        // TODO: use initial velocity of map.
        // TODO: clean this up pls.
        let mut last_velocity = 1.0;

        for sv in slider_velocities.into_iter().chain([final_sv]) {
            let last_pos = position_function.last().unwrap();

            let dt = sv.time - last_pos.time;
            
            let dy = last_velocity * dt as f64;

            let y = last_pos.position;

            last_velocity = sv.slider_velocity;

            position_function.push(PositionPoint {
                time: sv.time,
                position: y + dy,
            });
        }

        self.position_function = Arc::new(position_function);

        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.set_position_function(self.position_function.clone());
            }
        }
        for tl in self.timing_bars.iter_mut() {
            tl.set_position_function(self.position_function.clone());
        }
    }

    fn set_sv_mult_notes(&mut self) {
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.set_sv_mult(self.sv_mult)
            }
        }

        // update timing bars as well
        for t in self.timing_bars.iter_mut() {
            t.set_sv(self.sv_mult)
        }
    }
    
    async fn load_col_images(&mut self) {
        if let Some(settings) = &self.mania_skin_settings {
            let up_map = &settings.key_image;
            let down_map = &settings.key_image_d;
            for col in 0..self.column_count {
                let x = self.playfield.col_pos(col);

                // up image
                if let Some(path) = up_map.get(&col) {
                    if let Some(img) = SkinManager::get_texture(path, true).await {
                        let mut img = img.clone();
                        img.origin = Vector2::ZERO;
                        img.scale = self.playfield.note_size() / img.tex_size();
                        img.pos = Vector2::new(x, self.playfield.hit_y());

                        self.key_images_up.insert(col, img);
                    }
                }

                // down image
                if let Some(path) = down_map.get(&col) {
                    if let Some(img) = SkinManager::get_texture(path, true).await {
                        let mut img = img.clone();
                        img.origin = Vector2::ZERO;
                        img.scale = self.playfield.note_size() / img.tex_size();
                        img.pos = Vector2::new(x, self.playfield.hit_y());

                        self.key_images_down.insert(col, img);
                    }
                }

            }
        }
    }

    fn apply_new_playfield(&mut self, playfield: Arc<ManiaPlayfield>) {
        self.playfield = playfield.clone();
        
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.playfield_changed(playfield.clone());
            }
        }

        for timing_bar in self.timing_bars.iter_mut() {
            timing_bar.playfield_changed(playfield.clone());
        }
    }

    
    pub fn pos_at(position_function: &Arc<Vec<PositionPoint>>, time: f32, current_index: &mut usize) -> f64 {
        let (index, b) = position_function.iter().enumerate().skip(*current_index).find(|(_, p)| time < p.time)
            .unwrap_or_else(|| {
                (position_function.len() - 1, position_function.last().unwrap())
            });
        // warn!("time: {time}");
        *current_index = index;
        if index == 0 { return 0.0 }; // bad fix while neb fixes this
        let a = &position_function[index - 1];

        f64::lerp(a.position, b.position, ((time - a.time) / (b.time - a.time)) as f64)
    }



    fn add_hit_indicator(time: f32, column: usize, hit_value: &ManiaHitJudgments, column_count: u8, game_settings: &Arc<ManiaSettings>, playfield: &Arc<ManiaPlayfield>, manager: &mut IngameManager) {
        let color = hit_value.color();
        let image = None;
        // let (color, image) = match hit_value {
        //     Miss => (Color::RED, None),
        //     Okay | Good => (Color::LIME, None),
        //     Great | Marvelous => (Color::new(0.0, 0.7647, 1.0, 1.0), None),
        //     Perfect => Color::new(),
        // };

        let window_size = playfield.window_size;
        
        let total_width =column_count as f64 * playfield.column_width;
        let x_offset = playfield.x_offset + (window_size.x - total_width) / 2.0;

        let pos = Vector2::new(
            x_offset + playfield.x_offset + if game_settings.judgements_per_column {
                (playfield.column_width + playfield.column_spacing) * column as f64 + playfield.column_width / 2.0
            } else {
            ((playfield.column_width + playfield.column_spacing) * column_count as f64) / 2.0
            },

            if playfield.upside_down {playfield.hit_pos + game_settings.judgement_indicator_offset} else {window_size.y - playfield.hit_pos - game_settings.judgement_indicator_offset}
        );


        manager.add_judgement_indicator(BasicJudgementIndicator::new(
            pos, 
            time,
            -2.0,
            playfield.column_width / 2.0 * (2.0 / 3.0),
            color,
            image
        ))
    }


    fn keypress2col(key: KeyPress) -> Option<usize> {
        match key {
            KeyPress::Mania1 => Some(0),
            KeyPress::Mania2 => Some(1),
            KeyPress::Mania3 => Some(2),
            KeyPress::Mania4 => Some(3),
            KeyPress::Mania5 => Some(4),
            KeyPress::Mania6 => Some(5),
            KeyPress::Mania7 => Some(6),
            KeyPress::Mania8 => Some(7),
            KeyPress::Mania9 => Some(8),
            _ => None
        }
    }

}

#[async_trait]
impl GameMode for ManiaGame {
    async fn new(beatmap:&Beatmap, diff_calc_only: bool) -> TatakuResult<Self> {
        let metadata = beatmap.get_beatmap_meta();

        let game_settings = get_settings!().mania_settings.clone();
        let playfields = &game_settings.playfield_settings.clone();
        let auto_helper = ManiaAutoHelper::new();
        let window_size = WindowSize::get();

        let all_mania_skin_settings = &SkinManager::current_skin_config().await.mania_settings;
        let mut mania_skin_settings = None;
        let map_preferences = Database::get_beatmap_mode_prefs(&metadata.beatmap_hash, &"mania".to_owned()).await;
        
        // windows
        let hit_windows = vec![
            (ManiaHitJudgments::Marvelous, 0.0..18.0),
            (ManiaHitJudgments::Perfect, 18.0..43.0),
            (ManiaHitJudgments::Great, 43.0..76.0),
            (ManiaHitJudgments::Good, 76.0..106.0),
            (ManiaHitJudgments::Okay, 106.0..127.0),
            (ManiaHitJudgments::Miss, 127.0..164.0),
        ];

        let miss_window = hit_windows.last().unwrap().1.end;

        const DEFAULT_SNAP: Color = Color::SILVER;
        const SNAP_COLORS:&[(f32, Color)] = &[
            (0.0,        Color::RED),
            (1.0,        Color::RED),
            (1.0 / 2.0,  Color::BLUE),
            (1.0 / 3.0,  Color::PURPLE),
            (2.0 / 3.0,  Color::PURPLE),
            (1.0 / 4.0,  Color::YELLOW),
            (3.0 / 4.0,  Color::YELLOW),
            (1.0 / 6.0,  Color::PINK),
            (5.0 / 6.0,  Color::PINK),
            (1.0 / 8.0,  Color::ORANGE),
            (3.0 / 8.0,  Color::ORANGE),
            (5.0 / 8.0,  Color::ORANGE),
            (7.0 / 8.0,  Color::ORANGE),
            (1.0 / 12.0, Color::AQUA),
            (5.0 / 12.0, Color::AQUA),
            (7.0 / 12.0, Color::AQUA),
            (11.0 / 12.0, Color::AQUA),
            (1.0 / 16.0, Color::GREEN),
            (3.0 / 16.0, Color::GREEN),
            (5.0 / 16.0, Color::GREEN),
            (7.0 / 16.0, Color::GREEN),
            (9.0 / 16.0, Color::GREEN),
            (11.0 / 16.0, Color::GREEN),
            (13.0 / 16.0, Color::GREEN),
            (15.0 / 16.0, Color::GREEN),
        ];
        let timing_points = beatmap.get_timing_points();
        let get_color = |time| {
            let tp = timing_points.control_point_at(time);
        
            let offset = tp.time;
            let length = tp.beat_length;

            let threshold = 1.0 / length;

            let diff = time - offset;
            let snap = (diff / length) % 1.0;
            
            // temp/debug
            let mut closest_snap = (0.0, 99999.0);

            for (time, color) in SNAP_COLORS {
                let diff = (snap - *time).abs();
                if diff < 2.5 * threshold {
                    return *color;
                }
                if diff < closest_snap.1 {
                    closest_snap = (1.0 / *time, diff);
                }
            }
            
            // debug!("threshold: {}", threshold);
            // debug!("snap: {} - {:.1}", snap,  1.0 / snap);
            // debug!("lowestdiff: {:.5} {:.5}", closest_snap.0, closest_snap.1);

            DEFAULT_SNAP
        };


        let mut s = match beatmap {
            Beatmap::Osu(beatmap) => {
                let column_count = (beatmap.metadata.cs as u8).clamp(1, 9);
                let playfield = Arc::new(ManiaPlayfield::new(playfields[(column_count - 1) as usize].clone(), window_size.0, column_count));
                
                let get_hitsounds = |time, hitsound, hitsamples| {
                    let tp = timing_points.timing_point_at(time);
                    Hitsound::from_hitsamples(hitsound, hitsamples, true, tp)
                };

                let mut s = Self {
                    map_meta: metadata.clone(),
                    columns: Vec::new(),
                    column_indices:Vec::new(),
                    column_states: Vec::new(),
                    timing_bars: Vec::new(),
                    hit_windows,
                    miss_window,

                    position_function: Arc::new(Vec::new()),

                    end_time: 0.0,

                    sv_mult: map_preferences.scroll_speed as f64,
                    column_count,

                    auto_helper,
                    playfield,
                    mania_skin_settings,
                    map_preferences,
                    game_settings: Arc::new(game_settings),
                    key_images_up: HashMap::new(),
                    key_images_down: HashMap::new(),
                };

                for i in all_mania_skin_settings.iter() {
                    if i.keys == s.column_count {
                        s.mania_skin_settings = Some(Arc::new(i.clone()));
                        break;
                    }
                }

                // init defaults for the columsn
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.notes.iter() {
                    // if metadata.mode == "mania" {
                        let column = ((note.pos.x * s.column_count as f64 / 512.0).floor() as u8).min(column_count - 1);
                        let x = s.playfield.col_pos(column);
                        // warn!("{}, {:?}", note.hitsound, note.hitsamples);

                        s.columns[column as usize].push(Box::new(ManiaNote::new(
                            note.time,
                            column,
                            get_color(note.time),
                            x,
                            s.sv_mult,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                            get_hitsounds(note.time, note.hitsound, note.hitsamples.clone())
                        ).await));
                    // }
                }
                for hold in beatmap.holds.iter() {
                    let column = (hold.pos.x * s.column_count as f64 / 512.0).floor() as u8;
                    let x = s.playfield.col_pos(column);
                    s.columns[column as usize].push(Box::new(ManiaHold::new(
                        hold.time,
                        hold.end_time,
                        column,
                        get_color(hold.time),
                        x,
                        s.sv_mult,
                        s.playfield.clone(),
                        s.mania_skin_settings.clone(),
                        get_hitsounds(hold.time, hold.hitsound, hold.hitsamples.clone())
                    ).await));
                }

                for _slider in beatmap.sliders.iter() {
                    // let SliderDef {pos, time, slides, length, ..} = slider.to_owned();
                    // let time = time as u64;
                    
                    // let l = (length * 1.4) * slides as f64;
                    // let v2 = 100.0 * (beatmap.metadata.slider_multiplier as f64 * 1.4);
                    // let bl = beatmap.beat_length_at(time as f64, true);
                    // let end_time = time + (l / v2 * bl) as u64;
            
                    // let column = (pos.x * s.column_count as f64 / 512.0).floor() as u8;
                    // let x = s.col_pos(column);
                    // s.columns[column as usize].push(Box::new(ManiaHold::new(
                    //     time as u64,
                    //     end_time as u64,
                    //     x
                    // )));
                }
                for _spinner in beatmap.spinners.iter() {
                    // let SpinnerDef {time, end_time, ..} = spinner;
                    //TODO
                }

                s.integrate_velocity(beatmap.timing_points.iter().filter(|b| b.is_inherited()).map(|&b| SliderVelocity {
                    time: b.time,
                    slider_velocity: 100.0 / (-b.beat_length as f64) 
                }).collect());

                s
            }
            Beatmap::Quaver(beatmap) => {
                let column_count = beatmap.mode.into();
                for i in all_mania_skin_settings.iter() {
                    if i.keys == column_count {
                        mania_skin_settings = Some(Arc::new(i.clone()));
                    }
                }

                let playfield = Arc::new(ManiaPlayfield::new(playfields[(column_count - 1) as usize].clone(), window_size.0, column_count));

                let get_hitsounds = || {
                    vec![Hitsound::new_simple("normal-hitnormal")]
                };

                let mut s = Self {
                    map_meta: metadata.clone(),
                    columns: Vec::new(),
                    column_indices:Vec::new(),
                    column_states: Vec::new(),
                    timing_bars: Vec::new(),
                    hit_windows,
                    miss_window,

                    position_function: Arc::new(Vec::new()),
                    
                    end_time: 0.0,

                    sv_mult: map_preferences.scroll_speed as f64,
                    column_count,

                    auto_helper,
                    playfield,
                    mania_skin_settings,
                    map_preferences,
                    game_settings: Arc::new(game_settings),
                    
                    key_images_up:HashMap::new(),
                    key_images_down:HashMap::new(),
                };

                // init defaults for the columns
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.hit_objects.iter() {
                    let column = note.lane - 1;
                    let time = note.start_time;
                    let x = s.playfield.col_pos(column);

                    if let Some(end_time) = note.end_time {
                        s.columns[column as usize].push(Box::new(ManiaHold::new(
                            time,
                            end_time,
                            column,
                            get_color(time),
                            x,
                            s.sv_mult,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                            get_hitsounds()
                        ).await));
                    } else {
                        s.columns[column as usize].push(Box::new(ManiaNote::new(
                            time,
                            column,
                            get_color(time),
                            x,
                            s.sv_mult,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                            get_hitsounds()
                        ).await));
                    }
                }
                s.integrate_velocity(beatmap.slider_velocities.iter().map(|&x| x.into()).collect());

                s
            }
            Beatmap::Stepmania(beatmap) => {
                // stepmania maps are always 4k
                let column_count = 4;
                for i in all_mania_skin_settings.iter() {
                    if i.keys == column_count {
                        mania_skin_settings = Some(Arc::new(i.clone()));
                    }
                }

                let playfield = Arc::new(ManiaPlayfield::new(playfields[(column_count - 1) as usize].clone(), window_size.0, column_count));

                let get_hitsounds = || {
                    vec![Hitsound::new_simple("normal-hitnormal")]
                };

                let mut s = Self {
                    map_meta: metadata.clone(),
                    columns: Vec::new(),
                    column_indices:Vec::new(),
                    column_states: Vec::new(),
                    timing_bars: Vec::new(),
                    hit_windows,
                    miss_window,

                    position_function: Arc::new(Vec::new()),
                    
                    end_time: 0.0,

                    sv_mult: map_preferences.scroll_speed as f64,
                    column_count,

                    auto_helper,
                    playfield,
                    mania_skin_settings,
                    map_preferences,
                    game_settings: Arc::new(game_settings),
                    
                    key_images_up:HashMap::new(),
                    key_images_down:HashMap::new(),
                };

                // init defaults for the columns
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.chart_info.notes.iter() {
                    let column = note.column;
                    let time = note.start;
                    let x = s.playfield.col_pos(column);

                    if let Some(end_time) = note.end {
                        s.columns[column as usize].push(Box::new(ManiaHold::new(
                            time,
                            end_time,
                            column,
                            get_color(time),
                            x,
                            s.sv_mult,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                            get_hitsounds()
                        ).await));
                    } else {
                        s.columns[column as usize].push(Box::new(ManiaNote::new(
                            time,
                            column,
                            get_color(time),
                            x,
                            s.sv_mult,
                            s.playfield.clone(),
                            s.mania_skin_settings.clone(),
                            get_hitsounds()
                        ).await));
                    }
                }

                s.integrate_velocity(Vec::new());

                s
            }
            
            _ => return Err(BeatmapError::UnsupportedBeatmap.into()),
        };

        // get end time
        for col in s.columns.iter_mut() {
            col.sort_by(|a, b|a.time().partial_cmp(&b.time()).unwrap());
            if let Some(last_note) = col.iter().last() {
                s.end_time = s.end_time.max(last_note.end_time(0.0));
            }
        }
        s.end_time += 1000.0;
        if !diff_calc_only {
            s.reload_skin().await;
        }

        Ok(s)
    }

    async fn handle_replay_frame(&mut self, frame:ReplayFrame, time:f32, manager:&mut IngameManager) {
        if !manager.replaying {
            manager.replay.frames.push((time, frame));
            manager.outgoing_spectator_frame((time, SpectatorFrameData::ReplayFrame{frame}));
        }

        match frame {
            ReplayFrame::Press(key) => {
                manager.key_counter.key_down(key);
                let Some(col) = Self::keypress2col(key) else { return };
                // let hit_volume = get_settings!().get_effect_vol() * (manager.beatmap.timing_points[self.timing_point_index].volume as f32 / 100.0);

                // if theres no more notes to hit, return after playing the sound
                if self.column_indices[col] >= self.columns[col].len() {
                    // we need a hitsound though
                    let thing = self.columns[col].iter().last().unwrap();

                    manager.play_note_sound(thing.get_hitsound()).await;
                    // play_sound!(sound);
                    return;
                }
                let note = &mut self.columns[col][self.column_indices[col]];
                let note_time = note.time();
                *self.column_states.get_mut(col).unwrap() = true;

                if let Some(&judge) = manager.check_judgment(&self.hit_windows, time, note_time).await {
                    use ManiaHitJudgments::*;

                    // tell the note it was hit
                    note.hit(time);

                    // add the judgment
                    Self::add_hit_indicator(time, col, &judge, self.column_count, &self.game_settings, &self.playfield, manager);
                    
                    // play the hit sound

                    // we need a hitsound though
                    manager.play_note_sound(note.get_hitsound()).await;
                    // play_sound!(sound);

                    // incrememnt note index if this is not a slider
                    if note.note_type() != NoteType::Hold { self.next_note(col); }

                    // if this was a miss, check if we failed
                    if let Miss = judge {
                        if manager.health.is_dead() {
                            manager.fail();
                        }
                    }
                } else { // outside of any window, ignore
                    // play sound
                    let thing = &self.columns[col][self.column_indices[col]];

                    // play_sound!(sound);
                    manager.play_note_sound(thing.get_hitsound()).await;
                }
            }
            ReplayFrame::Release(key) => {
                manager.key_counter.key_up(key);
                let Some(col) = Self::keypress2col(key) else { return };

                *self.column_states.get_mut(col).unwrap() = false;
                if self.column_indices[col] >= self.columns[col].len() { return }

                let note = &mut self.columns[col][self.column_indices[col]];
                if time < note.time() - self.miss_window || time > note.end_time(self.miss_window) { return }
                note.release(time);

                if note.note_type() == NoteType::Hold {
                    let note_time = note.end_time(0.0);

                    if let Some(&judge) = manager.check_judgment(&self.hit_windows, time, note_time).await {
                        use ManiaHitJudgments::*;
    
                        // tell the note it was hit
                        note.hit(time);
    
                        // add the judgment
                        Self::add_hit_indicator(time, col, &judge, self.column_count, &self.game_settings, &self.playfield, manager);
                        
                        // // play the hit sound
                        // play_sound!(sound);
    
                        // increment note index 
                        self.next_note(col);
    
                        // if this was a miss, check if we failed
                        if let Miss = judge {
                            if manager.health.is_dead() {
                                manager.fail();
                            }
                        }
                    } else { // outside of any window, ignore
                        // play sound
                        let thing = &self.columns[col][self.column_indices[col]];

                        // play_sound!(sound);
                        manager.play_note_sound(thing.get_hitsound()).await;
                    }
                }
            }
        
            _ => {}
        }
    }


    async fn update(&mut self, manager:&mut IngameManager, time: f32) {
        if manager.current_mods.has_autoplay() {
            let mut frames = Vec::new();
            self.auto_helper.update(&self.columns, &mut self.column_indices, time, &mut frames);
            for frame in frames {
                self.handle_replay_frame(frame, time, manager).await
            }
        }

        // update notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {note.update(time).await}
        }

        // show score screen if map is over
        if time >= self.end_time {
            manager.completed = true;
            return;
        }

        // check if we missed the current note
        for col in 0..self.column_count as usize {
            if self.column_indices[col] >= self.columns[col].len() { continue; }
            let note = &self.columns[col][self.column_indices[col]];

            if note.end_time(self.miss_window) <= time {
                // TODO: do we need to check for holds?
                // if note.note_type() != NoteType::Hold || note.was_hit() {}

                let j = &ManiaHitJudgments::Miss;
                manager.add_judgment(j).await;
                Self::add_hit_indicator(time, col, j, self.column_count, &self.game_settings, &self.playfield, manager);
                self.next_note(col);
            }
        }   
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() {tb.update(time)}
    }
    
    async fn draw(&mut self, args:RenderArgs, manager:&mut IngameManager, list: &mut RenderableCollection) {
        let window_size = self.playfield.window_size;

        // playfield
        list.push(Rectangle::new(
            Color::new(0.0, 0.0, 0.0, 0.8),
            FIELD_DEPTH + 1.0,
            Vector2::new(self.playfield.col_pos(0), 0.0),
            Vector2::new(self.playfield.col_pos(self.column_count) - self.playfield.col_pos(0), window_size.y),
            Some(Border::new(if manager.current_timing_point().kiai {Color::YELLOW} else {Color::BLACK}, 1.2))
        ));


        // draw columns
        for col in 0..self.column_count {
            let x = self.playfield.col_pos(col);

            // column background
            list.push(Rectangle::new(
                Color::new(0.1, 0.1, 0.1, 0.8),
                FIELD_DEPTH,
                Vector2::new(x, 0.0),
                Vector2::new(self.playfield.column_width, window_size.y),
                Some(Border::new(Color::GREEN, 1.2))
            ));


            // hit area/button state for this col
            let map = if self.column_states[col as usize] {&self.key_images_down} else {&self.key_images_up};

            if let Some(img) = map.get(&col) {
                let mut img = img.clone();
                img.pos = Vector2::new(x, self.playfield.hit_y());

                list.push(img);
            } else {
                list.push(Rectangle::new(
                    if self.column_states[col as usize] {self.get_color(col)} else {Color::TRANSPARENT_WHITE},
                    HIT_AREA_DEPTH,
                    Vector2::new(x, self.playfield.hit_y()),
                    self.playfield.note_size(),
                    Some(Border::new(Color::RED, self.playfield.note_border_width))
                ));
            }
        }

        // draw notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() { note.draw(args, list).await}
        }
        // draw timing lines
        for tb in self.timing_bars.iter_mut() { tb.draw(args, list) }
    }

    fn skip_intro(&mut self, manager: &mut IngameManager) {
        // make sure we havent hit a note yet
        for &c in self.column_indices.iter() {if c > 0 {return}}

        let mut time = self.end_time;
        for col in self.columns.iter() {
            if let Some(note) = col.first() {
                time = time.min(note.time());
            }
        }

        // allow 2 seconds before the first note.
        time -= 2000.0;

        if time < 0.0 {return}
        if manager.time() >= time {return}

        if manager.lead_in_time > 0.0 {
            if time > manager.lead_in_time {
                time -= manager.lead_in_time - 0.01;
                manager.lead_in_time = 0.01;
            }
        }

        manager.song.set_position(time);
    }

    async fn reset(&mut self, beatmap:&Beatmap) {
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() {
                note.reset().await;
            }
        }
        for i in 0..self.columns.len() {
            self.column_indices[i] = 0;
            self.column_states[i] = false;
        }

        // setup timing bars
        //TODO: it would be cool if we didnt actually need timing bar objects, and could just draw them
        let x = self.playfield.col_pos(0);
        if self.timing_bars.len() == 0 {
            let tps = beatmap.get_timing_points();
            // load timing bars
            let parent_tps = tps.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = beatmap.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            let bar_width = (self.playfield.column_width + self.playfield.column_spacing) * self.column_count as f64 - self.playfield.column_spacing;

            loop {
                // if theres a bpm change, adjust the current time to that of the bpm change
                let measure = if tp_index < parent_tps.len() { parent_tps[tp_index].meter } else { 4 };
                let next_bar_time = beatmap.beat_length_at(time, false) * measure as f32;

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                let mut bar = TimingBar::new(time, bar_width, x, self.playfield.clone());
                bar.set_position_function(self.position_function.clone());
                bar.set_sv(self.sv_mult);
                self.timing_bars.push(bar);

                if tp_index < parent_tps.len() && parent_tps[tp_index].time <= time + next_bar_time {
                    time = parent_tps[tp_index].time;
                    tp_index += 1;
                    continue;
                }

                // why isnt this accounting for bpm changes? because the bpm change doesnt allways happen inline with the bar idiot
                time += next_bar_time;
                if time >= self.end_time || time.is_nan() {break}
            }

            debug!("created {} timing bars", self.timing_bars.len());
        } else {
            for t in self.timing_bars.iter_mut() {
                t.reset();
            }
        }
    }

    
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        let playfield = Arc::new(ManiaPlayfield::new(self.game_settings.playfield_settings[(self.column_count - 1) as usize].clone(), window_size.0, self.column_count));
        self.apply_new_playfield(playfield);
    }


    async fn fit_to_area(&mut self, pos: Vector2, size: Vector2) {
        let mut playfield = ManiaPlayfield::new(
            self.game_settings.playfield_settings[(self.column_count - 1) as usize].clone(), 
            size, 
            self.column_count
        );

        playfield.settings.x_offset = pos.x;

        // if playfield.upside_down {
        //     playfield.settings.hit_pos -= pos.y
        // } else {
        //     playfield.settings.hit_pos += pos.y
        // }
        

        self.apply_new_playfield(Arc::new(playfield));
    }

    
    async fn force_update_settings(&mut self, _settings: &Settings) {}
    
    async fn reload_skin(&mut self) {
        // reload skin settings
        let all_mania_skin_settings = &SkinManager::current_skin_config().await.mania_settings;
        for i in all_mania_skin_settings.iter() {
            if i.keys == self.column_count {
                self.mania_skin_settings = Some(Arc::new(i.clone()));
                break;
            }
        }
        
        for c in self.columns.iter_mut() {
            for n in c.iter_mut() {
                n.reload_skin().await;
            }
        }
        
        self.load_col_images().await;
    }

    async fn apply_mods(&mut self, _mods: Arc<ModManager>) {

    }
}


#[async_trait]
impl GameModeInput for ManiaGame {

    async fn key_down(&mut self, key:piston::Key, manager:&mut IngameManager) {
        // check sv change keys
        if key == Key::F4 || key == Key::F3 {
            if key == Key::F4 {
                self.sv_mult += self.game_settings.sv_change_delta as f64;
            } else {
                self.sv_mult -= self.game_settings.sv_change_delta as f64;
            }
            self.map_preferences.scroll_speed = self.sv_mult as f32;

            self.set_sv_mult_notes();

            return;
        }

        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }


        let settings = get_settings!();
        let mut game_key = KeyPress::RightDon;

        let keys = &settings.mania_settings.keys[(self.column_count-1) as usize];
        let base_key = KeyPress::Mania1 as u8;
        for col in 0..self.column_count as usize {
            let k = keys[col];
            if k == key {
                game_key = ((col + base_key as usize) as u8).into();
                break;
            }
        }
        if game_key == KeyPress::RightDon {return}
        let time = manager.time();
        self.handle_replay_frame(ReplayFrame::Press(game_key), time, manager).await;
    }
    
    async fn key_up(&mut self, key:piston::Key, manager:&mut IngameManager) {
        // dont accept key input when autoplay is enabled, or a replay is being watched
        if manager.current_mods.has_autoplay() || manager.replaying {
            return;
        }

        let settings = get_settings!();
        let mut game_key = KeyPress::RightDon;

        let keys = &settings.mania_settings.keys[(self.column_count-1) as usize];
        let base_key = KeyPress::Mania1 as u8;
        for col in 0..self.column_count as usize {
            let k = keys[col];
            if k == key {
                game_key = ((col + base_key as usize) as u8).into();
                break;
            }
        }
        if game_key == KeyPress::RightDon {return}
        let time = manager.time();

        self.handle_replay_frame(ReplayFrame::Release(game_key), time, manager).await;
    }

}

#[async_trait]
impl GameModeProperties for ManiaGame {
    fn playmode(&self) -> PlayMode { "mania".to_owned() }

    fn end_time(&self) -> f32 { self.end_time }
    
    fn get_possible_keys(&self) -> Vec<(KeyPress, &str)> {
        let mut list = Vec::new();
        for i in 0..self.column_count {
            match i {
                0 => list.push((KeyPress::Mania1, "K1")),
                1 => list.push((KeyPress::Mania2, "K2")),
                2 => list.push((KeyPress::Mania3, "K3")),
                3 => list.push((KeyPress::Mania4, "K4")),
                4 => list.push((KeyPress::Mania5, "K5")),
                5 => list.push((KeyPress::Mania6, "K6")),
                6 => list.push((KeyPress::Mania7, "K7")),
                7 => list.push((KeyPress::Mania8, "K8")),
                8 => list.push((KeyPress::Mania9, "K9")),
                _ => {}
            }
        }
        
        list
    }

    fn timing_bar_things(&self) -> Vec<(f32,Color)> {
        self.hit_windows
            .iter()
            .map(|(j, w) | {
                (w.end, j.color())
            })
            .collect()
    }

    async fn get_ui_elements(&self, window_size: Vector2, ui_elements: &mut Vec<UIElement>) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };


        let start_x = self.playfield.col_pos(0);
        let width = self.playfield.col_pos(self.column_count) - start_x;

        let combo_bounds = Rectangle::bounds_only(
            Vector2::ZERO,
            Vector2::new(width, 30.0)
        );
        
        // combo
        ui_elements.push(UIElement::new(
            &get_name("combo".to_owned()),
            Vector2::new(start_x, window_size.y * (1.0/3.0)),
            ComboElement::new(combo_bounds).await
        ).await);

        // Leaderboard
        ui_elements.push(UIElement::new(
            &get_name("leaderboard".to_owned()),
            Vector2::with_y(window_size.y / 3.0),
            LeaderboardElement::new().await
        ).await);
        
    }

}

// when the game is dropped, save settings
// this is better than saving the update every time the values change
impl Drop for ManiaGame {
    fn drop(&mut self) {
        Database::save_beatmap_mode_prefs(&self.map_meta.beatmap_hash, &"mania".to_owned(), &self.map_preferences);
    }
}



// TODO: document whatever the hell is happening here
struct ManiaAutoHelper {
    states: Vec<AutoplayColumnState>,
}
impl ManiaAutoHelper {
    fn new() -> Self {
        Self {
            states: Vec::new(),
        }
    }

    fn get_keypress(col: usize) -> KeyPress {
        let base_key = KeyPress::Mania1 as u8;
        ((col + base_key as usize) as u8).into()
    }

    fn update(&mut self, columns: &Vec<Vec<Box<dyn ManiaHitObject>>>, column_indices: &mut Vec<usize>, time: f32, list: &mut Vec<ReplayFrame>) {
        if self.states.len() != columns.len() {
            let new_len = columns.len();
            self.states.resize(new_len, AutoplayColumnState::default());
            // self.notes_hit.resize(new_len, Vec::new());
        }

        for c in 0..columns.len() {
            let state = &mut self.states[c];
            if state.pressed && time > state.release_time {
                list.push(ReplayFrame::Release(Self::get_keypress(c)));
                state.pressed = false;
            }

            if column_indices[c] >= columns[c].len() {continue}

            // catch up??
            for i in column_indices[c]..columns[c].len() {
                let note = &columns[c][i];
                if time > note.end_time(100.0) && !note.was_hit() {
                    column_indices[c] += 1;
                } else {
                    break;
                }
            }

            if column_indices[c] >= columns[c].len() { continue }
            let note = &columns[c][column_indices[c]];
            if time >= note.time() && !note.was_hit() {
                // if the key is already down, dont press it again
                // if timer.0 == note.end_time(15.0) && 
                if state.pressed { continue }

                // press the key, and hold it until the note's end time
                list.push(ReplayFrame::Press(Self::get_keypress(c)));
                state.pressed = true;
                if note.note_type() == NoteType::Hold {
                    state.release_time = note.end_time(0.0);
                } else {
                    state.release_time = note.end_time(50.0);
                }
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
struct AutoplayColumnState {
    pressed: bool,
    release_time: f32
}


#[derive(Clone, Copy, Debug, Default)]
pub struct SliderVelocity {
    /// Start time of the timing section, in milliseconds from the beginning of the beatmap's audio. The end of the timing section is the next timing point's time (or never, if this is the last timing point).
    pub time: f32,
    
    /// Velocity multiplier
    pub slider_velocity: f64,
}
impl From<QuaverSliderVelocity> for SliderVelocity {
    fn from(s: QuaverSliderVelocity) -> Self {
        Self {
            time: s.start_time,
            slider_velocity: s.multiplier,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PositionPoint {
    pub time: f32,
    pub position: f64
}

impl Default for PositionPoint {
    fn default() -> Self {
        Self {
            time: -LEAD_IN_TIME,
            position: -LEAD_IN_TIME as f64,
        }
    }
}


#[derive(Clone)]
pub struct ManiaPlayfield {
    settings: ManiaPlayfieldSettings,
    pub window_size: Vector2,
    col_count: u8
}
impl ManiaPlayfield {
    pub fn new(settings: ManiaPlayfieldSettings, window_size: Vector2, col_count: u8) -> Self {
        Self {
            settings, 
            window_size,
            col_count
        }
    }

    pub fn hit_y(&self) -> f64 {
        if self.upside_down {
            self.hit_pos
        } else {
            self.window_size.y - self.hit_pos
        }
    }
    pub fn col_pos(&self, col: u8) -> f64 {
        let total_width = self.col_count as f64 * self.column_width;
        let x_offset = self.x_offset + (self.window_size.x - total_width) / 2.0;

        x_offset + self.x_offset + (self.column_width + self.column_spacing) * col as f64
    }
}


impl Deref for ManiaPlayfield {
    type Target = ManiaPlayfieldSettings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}
