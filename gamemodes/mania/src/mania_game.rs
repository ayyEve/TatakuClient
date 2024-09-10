/**
 * Mania game mode
 * Authored by ayyEve
 * scroll velocity by Nebula
 */

use crate::prelude::*;

const OSU_SIZE: Vector2 = Vector2::new(640.0, 480.0);

pub struct ManiaGame {
    map_meta: Arc<BeatmapMeta>,
    // lists
    pub columns: Vec<Vec<Box<dyn ManiaHitObject>>>,
    timing_bars: Vec<TimingBar>,
    hit_windows: Vec<(HitJudgment, Range<f32>)>,
    miss_window: f32,

    position_function: Arc<Vec<PositionPoint>>,

    // list indices
    column_indices: Vec<usize>,
    /// true if held
    column_states: Vec<bool>,

    end_time: f32,
    sv_mult: f32,
    column_count: u8,

    auto_helper: ManiaAutoHelper,
    playfield: Arc<ManiaPlayfield>,

    game_settings: Arc<ManiaSettings>,

    mania_skin_settings: Option<Arc<ManiaSkinSettings>>,
    map_preferences: BeatmapPlaymodePreferences,

    key_images_up: HashMap<u8, Image>,
    key_images_down: HashMap<u8, Image>,
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
                position: self.end_time,
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
            
            let dy = last_velocity * dt;

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
    
    // #[cfg(feature="graphics")]
    async fn load_col_images(&mut self, source: &TextureSource, skin_manager: &mut dyn SkinProvider) {
        let Some(settings) = &self.mania_skin_settings else { return };
        self.key_images_down.clear();
        self.key_images_up.clear();
        
        let note_size = self.playfield.note_size();
        let y = self.playfield.hit_y() + note_size.y;

        for col in 0..self.column_count {
            let x = self.playfield.col_pos(col);

            for (path_map, image_map) in [
                // up images
                (&settings.key_image, &mut self.key_images_up),
                // down images
                (&settings.key_image_d, &mut self.key_images_down),
            ] {
                let Some(path) = path_map.get(&col) else { continue };
                let Some(mut img) = skin_manager.get_texture(path, source, SkinUsage::Beatmap, true).await else { continue };
                self.playfield.column_image(&mut img);
                img.pos = Vector2::new(x, y);

                image_map.insert(col, img);
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

        let note_size = self.playfield.note_size();
        let y = playfield.hit_y() + note_size.y;
        for col in 0..self.column_count {
            let x = self.playfield.col_pos(col);

            for image_map in [&mut self.key_images_down, &mut self.key_images_up] {
                let Some(img) = image_map.get_mut(&col) else { continue };
                self.playfield.column_image(img);

                let tex_size = img.tex_size();
                // img.origin = Vector2::new(0.0, tex_size.y-playfield.note_yoffset);
                // img.origin = Vector2::with_y(tex_size.y);

                img.scale = Vector2::ONE * (note_size.x / tex_size.x);
                img.pos = Vector2::new(x, y);
            }
        }

    }

    
    pub fn pos_at(position_function: &Arc<Vec<PositionPoint>>, time: f32, current_index: &mut usize) -> f32 {
        let (index, b) = position_function.iter().enumerate().skip(*current_index).find(|(_, p)| time < p.time)
            .unwrap_or_else(|| {
                (position_function.len() - 1, position_function.last().unwrap())
            });
        // warn!("time: {time}");
        *current_index = index;
        if index == 0 { return 0.0 }; // bad fix while neb fixes this
        let a = &position_function[index - 1];

        f32::lerp(a.position, b.position, (time - a.time) / (b.time - a.time))
    }



    fn add_hit_indicator(
        column: usize, 
        hit_value: &HitJudgment, 
        column_count: u8, 
        game_settings: &Arc<ManiaSettings>, 
        playfield: &Arc<ManiaPlayfield>, 
        state: &mut GameplayStateForUpdate<'_>
    ) {
        let color = hit_value.color;
        let image = None;
        // let (color, image) = match hit_value {
        //     Miss => (Color::RED, None),
        //     Okay | Good => (Color::LIME, None),
        //     Great | Marvelous => (Color::new(0.0, 0.7647, 1.0, 1.0), None),
        //     Perfect => Color::new(),
        // };

        let bounds = playfield.bounds;
        
        let total_width = column_count as f32 * playfield.column_width;
        let x_offset = playfield.x_offset + (bounds.size.x - total_width) / 2.0;

        let pos = bounds.pos + Vector2::new(
            x_offset + playfield.x_offset + if game_settings.judgements_per_column {
                (playfield.column_width + playfield.column_spacing) * column as f32 + playfield.column_width / 2.0
            } else {
                ((playfield.column_width + playfield.column_spacing) * column_count as f32) / 2.0
            },

            if playfield.upside_down {playfield.hit_pos + game_settings.judgement_indicator_offset} else {bounds.size.y - playfield.hit_pos - game_settings.judgement_indicator_offset}
        );


        state.add_indicator(BasicJudgementIndicator::new(
            pos, 
            state.time,
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

    async fn draw_notes(&mut self, time: f32, list: &mut RenderableCollection) {
        // draw timing bars
        for tb in self.timing_bars.iter_mut() { tb.draw(list) }

        // draw notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() { note.draw(time, list).await }
        }
    }

    async fn draw_columns(&mut self, bounds: Bounds, list: &mut RenderableCollection) {

        for col in 0..self.column_count {
            let x = self.playfield.col_pos(col);

            // column background
            list.push(Rectangle::new(
                Vector2::new(x, bounds.pos.y),
                Vector2::new(self.playfield.column_width, bounds.size.y),
                Color::new(0.1, 0.1, 0.1, 0.8),
                Some(Border::new(Color::GREEN, 1.2))
            ));

            // hit area/button state for this col
            let map = if self.column_states[col as usize] { &self.key_images_down } else { &self.key_images_up };

            if let Some(img) = map.get(&col) {
                // let mut img = img.clone();
                // img.pos = Vector2::new(x, self.playfield.hit_y());
                
                list.push(img.clone());
            } else {
                list.push(Rectangle::new(
                    Vector2::new(x, self.playfield.hit_y()),
                    self.playfield.note_size(),
                    if self.column_states[col as usize] { self.get_color(col) } else { Color::TRANSPARENT_WHITE },
                    Some(Border::new(Color::RED, self.playfield.note_border_width))
                ));
            }
        }
    }
    
}

#[async_trait]
impl GameMode for ManiaGame {
    async fn new(beatmap:&Beatmap, _: bool, settings: &Settings) -> TatakuResult<Self> {
        let metadata = beatmap.get_beatmap_meta();

        let game_settings = settings.mania_settings.clone();
        let playfields = &game_settings.playfield_settings.clone();
        let auto_helper = ManiaAutoHelper::new();
        let window_size = WindowSize::get();

        // let all_mania_skin_settings = &SkinManager::skin().await.mania_settings;
        let map_preferences = Database::get_beatmap_mode_prefs(metadata.beatmap_hash, &"mania".to_owned()).await;
        
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
                
                let get_hitsounds = |time, hitsound, hitsamples| {
                    let tp = timing_points.timing_point_at(time);
                    Hitsound::from_hitsamples(hitsound, hitsamples, true, tp)
                };
                // for i in all_mania_skin_settings.iter() {
                //     if i.keys == column_count {
                //         mania_skin_settings = Some(Arc::new(i.clone()));
                //         break;
                //     }
                // }

                let playfield = Arc::new(ManiaPlayfield::new(
                    playfields[(column_count - 1) as usize].clone(), 
                    Bounds::new(Vector2::ZERO, window_size.0), 
                    column_count,
                    0.0,
                    // mania_skin_settings.as_ref().map(|s|OSU_SIZE.y - s.hit_position).unwrap_or_default()
                ));


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

                    sv_mult: map_preferences.scroll_speed,
                    column_count,

                    auto_helper,
                    playfield,
                    mania_skin_settings: None,
                    map_preferences,
                    game_settings: Arc::new(game_settings),
                    key_images_up: HashMap::new(),
                    key_images_down: HashMap::new(),
                };


                // init defaults for the columsn
                for _col in 0..s.column_count {
                    s.columns.push(Vec::new());
                    s.column_indices.push(0);
                    s.column_states.push(false);
                }

                // add notes
                for note in beatmap.notes.iter() {
                    // if metadata.mode == "mania" {
                        let column = ((note.pos.x * s.column_count as f32 / 512.0).floor() as u8).min(column_count - 1);
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
                    let column = (hold.pos.x * s.column_count as f32 / 512.0).floor() as u8;
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
                    slider_velocity: 100.0 / (-b.beat_length) 
                }).collect());

                s
            }
            Beatmap::Quaver(beatmap) => {
                let column_count = beatmap.mode.into();
                // for i in all_mania_skin_settings.iter() {
                //     if i.keys == column_count {
                //         mania_skin_settings = Some(Arc::new(i.clone()));
                //     }
                // }

                let playfield = Arc::new(ManiaPlayfield::new(
                    playfields[(column_count - 1) as usize].clone(), 
                    Bounds::new(Vector2::ZERO, window_size.0), 
                    column_count,
                    0.0
                    // mania_skin_settings.as_ref().map(|s|OSU_SIZE.y - s.hit_position).unwrap_or_default(),
                ));

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

                    sv_mult: map_preferences.scroll_speed,
                    column_count,

                    auto_helper,
                    playfield,
                    mania_skin_settings: None,
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
                            None,
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
                            None,
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
                // for i in all_mania_skin_settings.iter() {
                //     if i.keys == column_count {
                //         mania_skin_settings = Some(Arc::new(i.clone()));
                //     }
                // }

                let playfield = Arc::new(ManiaPlayfield::new(
                    playfields[(column_count - 1) as usize].clone(), 
                    Bounds::new(Vector2::ZERO, window_size.0), 
                    column_count,
                    0.0
                    // mania_skin_settings.as_ref().map(|s|OSU_SIZE.y - s.hit_position).unwrap_or_default()
                ));

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

                    sv_mult: map_preferences.scroll_speed,
                    column_count,

                    auto_helper,
                    playfield,
                    mania_skin_settings: None,
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

        Ok(s)
    }

    async fn handle_replay_frame<'a>(
        &mut self, 
        frame: ReplayFrame, 
        state: &mut GameplayStateForUpdate<'a>
    ) {
        match frame.action {
            ReplayAction::Press(key) => {
                let Some(col) = Self::keypress2col(key) else { return };
                // let hit_volume = Settings::get().get_effect_vol() * (manager.beatmap.timing_points[self.timing_point_index].volume as f32 / 100.0);

                // if theres no more notes to hit, return after playing the sound
                if self.column_indices[col] >= self.columns[col].len() {
                    // we need a hitsound though
                    let thing = self.columns[col].iter().last().unwrap();

                    state.play_note_sound(thing.get_hitsound().clone());

                    // play_sound!(sound);
                    return;
                }
                let note = &mut self.columns[col][self.column_indices[col]];
                let note_time = note.time();
                *self.column_states.get_mut(col).unwrap() = true;

                if let Some(&judge) = state.check_judgment(&self.hit_windows, frame.time, note_time).await {
                    // use ManiaHitJudgments::*;

                    // tell the note it was hit
                    note.hit(frame.time);

                    // add the judgment
                    Self::add_hit_indicator(
                        col, 
                        &judge, 
                        self.column_count, 
                        &self.game_settings, 
                        &self.playfield, 
                        state
                    );
                    
                    // play the hit sound
                    state.play_note_sound(note.get_hitsound().clone());
                    // play_sound!(sound);

                    // incrememnt note index if this is not a slider
                    if note.note_type() != NoteType::Hold { self.next_note(col); }


                    // TODO: is this necessary? 
                    // this should be handled by the gameplay manager, not sure why its here
                    
                    // // if this was a miss, check if we failed
                    // if judge == ManiaHitJudgments::Miss {
                    //     if manager.health.is_dead() {
                    //         manager.fail();
                    //     }
                    // }
                } else { // outside of any window, ignore
                    // play sound
                    let thing = &self.columns[col][self.column_indices[col]];

                    // play_sound!(sound);
                    state.play_note_sound(thing.get_hitsound().clone());
                }
            }
            ReplayAction::Release(key) => {
                let Some(col) = Self::keypress2col(key) else { return };

                *self.column_states.get_mut(col).unwrap() = false;
                if self.column_indices[col] >= self.columns[col].len() { return }

                let note = &mut self.columns[col][self.column_indices[col]];
                if frame.time < note.time() - self.miss_window || frame.time > note.end_time(self.miss_window) { return }
                note.release(frame.time);

                if note.note_type() == NoteType::Hold {
                    let note_time = note.end_time(0.0);

                    if let Some(&judge) = state.check_judgment(&self.hit_windows, frame.time, note_time).await {
    
                        // tell the note it was hit
                        note.hit(frame.time);
    
                        // add the judgment
                        Self::add_hit_indicator(
                            col, 
                            &judge, 
                            self.column_count, 
                            &self.game_settings, 
                            &self.playfield, 
                            state
                        );
                        
                        // // play the hit sound
                        // play_sound!(sound);
    
                        // increment note index 
                        self.next_note(col);
    
                        // TODO: again, is this necessary?

                        // // if this was a miss, check if we failed
                        // if judge == ManiaHitJudgments::Miss && manager.health.is_dead() {
                        //     manager.fail();
                        // }
                    } else { // outside of any window, ignore
                        // play sound
                        let thing = &self.columns[col][self.column_indices[col]];

                        // play_sound!(sound);
                        state.play_note_sound(thing.get_hitsound().clone());
                    }
                }
            }
        
            _ => {}
        }
    }


    async fn update<'a>(
        &mut self, 
        state: &mut GameplayStateForUpdate<'a>
    ) {
        if state.mods.has_autoplay() {
            let mut frames = Vec::new();
            self.auto_helper.update(&self.columns, &mut self.column_indices, state.time, &mut frames);
            for frame in frames {
                self.handle_replay_frame(ReplayFrame::new(state.time, frame), state).await
            }
        }

        // update notes
        for col in self.columns.iter_mut() {
            for note in col.iter_mut() { note.update(state.time).await }
        }

        // dont continue if map is over
        if state.time >= self.end_time {
            if !state.complete() {
                state.add_action(GamemodeAction::MapComplete);
                // manager.completed = true;
            }
            return;
        }

        // check if we missed the current note
        for col in 0..self.column_count as usize {
            if self.column_indices[col] >= self.columns[col].len() { continue; }
            let note = &self.columns[col][self.column_indices[col]];

            if note.end_time(self.miss_window) <= state.time {
                // TODO: do we need to check for holds?
                // if note.note_type() != NoteType::Hold || note.was_hit() {}

                let j = ManiaHitJudgments::Miss;
                state.add_judgment(j);
                Self::add_hit_indicator(col, &j, self.column_count, &self.game_settings, &self.playfield, state);
                self.next_note(col);
            }
        }
        
        // TODO: might move tbs to a (time, speed) tuple
        for tb in self.timing_bars.iter_mut() { tb.update(state.time) }
    }
    
    async fn draw<'a>(&mut self, state:GameplayStateForDraw<'a>, list: &mut RenderableCollection) {
        let bounds = self.playfield.bounds;

        // playfield
        list.push(Rectangle::new(
            Vector2::new(self.playfield.col_pos(0), bounds.pos.y),
            Vector2::new(self.playfield.total_width, bounds.size.y),
            Color::new(0.0, 0.0, 0.0, 0.8),
            Some(Border::new(if state.current_timing_point.kiai { Color::YELLOW } else { Color::BLACK }, 1.2))
        ));


        // draw columns
        self.draw_columns(bounds, list).await;

        // draw notes and timing bars
        self.draw_notes(state.time, list).await;
    }

    fn skip_intro(&mut self, game_time: f32) -> Option<f32> {
        // make sure we havent hit a note yet
        for &c in self.column_indices.iter() { if c > 0 { return None } }

        let mut time = self.end_time;
        for col in self.columns.iter() {
            if let Some(note) = col.first() {
                time = time.min(note.time());
            }
        }

        // allow 2 seconds before the first note.
        time -= 2000.0;

        if time < 0.0 { return None }
        if game_time >= time { return None }

        // if manager.lead_in_time > 0.0 {
        //     if time > manager.lead_in_time {
        //         time -= manager.lead_in_time - 0.01;
        //         manager.lead_in_time = 0.01;
        //     }
        // }

        Some(time)
    }

    async fn reset(&mut self, beatmap:&Beatmap) {
        let timing_points = TimingPointHelper::new(beatmap.get_timing_points(), beatmap.slider_velocity());

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
        if self.timing_bars.len() == 0 {
            // load timing bars
            let parent_tps = timing_points.iter().filter(|t|!t.is_inherited()).collect::<Vec<&TimingPoint>>();
            let mut time = parent_tps[0].time;
            let mut tp_index = 0;
            let step = timing_points.beat_length_at(time, false);
            time %= step; // get the earliest bar line possible

            let bar_width = (self.playfield.column_width + self.playfield.column_spacing) * self.column_count as f32 - self.playfield.column_spacing;

            loop {
                // if theres a bpm change, adjust the current time to that of the bpm change
                let measure = if tp_index < parent_tps.len() { parent_tps[tp_index].meter } else { 4 };
                let next_bar_time = timing_points.beat_length_at(time, false) * measure as f32;

                // edge case for aspire maps
                if next_bar_time.is_nan() || next_bar_time == 0.0 {
                    break;
                }

                // add timing bar at current time
                let mut bar = TimingBar::new(time, bar_width, self.playfield.clone());
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
        let playfield = Arc::new(ManiaPlayfield::new(
            self.game_settings.playfield_settings[(self.column_count - 1) as usize].clone(), 
            Bounds::new(Vector2::ZERO, window_size.0), 
            self.column_count,
            self.mania_skin_settings.as_ref().map(|s|OSU_SIZE.y - s.hit_position).unwrap_or_default()
        ));
        self.apply_new_playfield(playfield);
    }


    async fn fit_to_area(&mut self, bounds: Bounds) {
        let mut playfield = ManiaPlayfield::new(
            self.game_settings.playfield_settings[(self.column_count - 1) as usize].clone(), 
            bounds, 
            self.column_count,
            self.mania_skin_settings.as_ref().map(|s|OSU_SIZE.y - s.hit_position).unwrap_or_default()
        );

        playfield.settings.x_offset = bounds.pos.x;

        // if playfield.upside_down {
        //     playfield.settings.hit_pos -= pos.y
        // } else {
        //     playfield.settings.hit_pos += pos.y
        // }
        

        self.apply_new_playfield(Arc::new(playfield));
    }

    
    async fn force_update_settings(&mut self, _settings: &Settings) {}
    
    async fn reload_skin(&mut self, beatmap_path: &String, skin_manager: &mut dyn SkinProvider) -> TextureSource {
        let source = TextureSource::Beatmap(beatmap_path.clone()); // TODO: add setting option

        // reload skin settings
        let all_mania_skin_settings = &skin_manager.skin().mania_settings;
        for i in all_mania_skin_settings.iter() {
            if i.keys == self.column_count {
                self.mania_skin_settings = Some(Arc::new(i.clone()));
                break;
            }
        }
        
        for c in self.columns.iter_mut() {
            for n in c.iter_mut() {
                n.reload_skin(&source, skin_manager).await;
            }
        }
        
        self.load_col_images(&source, skin_manager).await;

        source
    }

    async fn apply_mods(&mut self, _mods: Arc<ModManager>) { }

    async fn beat_happened(&mut self, pulse_length: f32) {
        self.columns.iter_mut().flatten().for_each(|n|n.beat_happened(pulse_length))
    }
    async fn kiai_changed(&mut self, is_kiai: bool) {
        self.columns.iter_mut().flatten().for_each(|n|n.kiai_changed(is_kiai))
    }
}


// #[cfg(feature="graphics")]
#[async_trait]
impl GameModeInput for ManiaGame {

    async fn key_down(&mut self, key:Key) -> Option<ReplayAction> {
        // check sv change keys
        if key == Key::F4 || key == Key::F3 {
            if key == Key::F4 {
                self.sv_mult += self.game_settings.sv_change_delta;
            } else {
                self.sv_mult -= self.game_settings.sv_change_delta;
            }
            self.map_preferences.scroll_speed = self.sv_mult;

            self.set_sv_mult_notes();

            return None;
        }

        let mut game_key = KeyPress::RightDon;
    
        let keys = &self.game_settings.keys[(self.column_count-1) as usize];
        let base_key = KeyPress::Mania1 as u8;
        for col in 0..self.column_count as usize {
            let k = keys[col];
            if k == key {
                game_key = ((col + base_key as usize) as u8).into();
                break;
            }
        }
        
        if game_key == KeyPress::RightDon { return None }
        Some(ReplayAction::Press(game_key))
    }
    
    async fn key_up(&mut self, key:Key) -> Option<ReplayAction> {
        let mut game_key = KeyPress::RightDon;

        let keys = &self.game_settings.keys[(self.column_count-1) as usize];
        let base_key = KeyPress::Mania1 as u8;
        for col in 0..self.column_count as usize {
            let k = keys[col];
            if k == key {
                game_key = ((col + base_key as usize) as u8).into();
                break;
            }
        }

        if game_key == KeyPress::RightDon { return None } 
        Some(ReplayAction::Release(game_key))
    }

}


// #[cfg(not(feature="graphics"))]
// impl GameModeInput for ManiaGame {}



#[async_trait]
impl GameModeProperties for ManiaGame {
    fn playmode(&self) -> Cow<'static, str> { Cow::Borrowed("mania") }
    fn get_info(&self) -> GameModeInfo { crate::GAME_INFO }

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

    fn timing_bar_things(&self) -> Vec<(f32, Color)> {
        self.hit_windows
            .iter()
            .map(|(j, w)| (w.end, j.color))
            .collect()
    }

    async fn get_ui_elements(&self, window_size: Vector2, ui_elements: &mut Vec<UIElement>) {
        let playmode = self.playmode();
        let get_name = |name| {
            format!("{playmode}_{name}")
        };


        let start_x = self.playfield.col_pos(0);
        let width = self.playfield.col_pos(self.column_count) - start_x;

        let combo_bounds = Bounds::new(
            Vector2::ZERO,
            Vector2::new(width, 30.0)
        );
        
        // combo
        ui_elements.push(UIElement::new(
            &get_name("combo".to_owned()),
            Vector2::new(start_x, window_size.y * (1.0/3.0)),
            ComboElement::new(combo_bounds).await
        ).await);

        // TODO: !!!
        // // Leaderboard
        // ui_elements.push(UIElement::new(
        //     &get_name("leaderboard".to_owned()),
        //     Vector2::with_y(window_size.y / 3.0),
        //     LeaderboardElement::new().await
        // ).await);
        
    }

}

// when the game is dropped, save settings
// this is better than saving the update every time the values change
impl Drop for ManiaGame {
    fn drop(&mut self) {
        Database::save_beatmap_mode_prefs(self.map_meta.beatmap_hash, &"mania".to_owned(), &self.map_preferences);
    }
}

