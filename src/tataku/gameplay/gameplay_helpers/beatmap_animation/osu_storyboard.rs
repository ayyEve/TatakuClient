use crate::prelude::*;

const GAME_SIZE: Vector2 = Vector2::new(640.0, 480.0);

pub struct OsuStoryboard {
    scaling_helper: Arc<ScalingHelper>,
    settings: OsuSettings,

    elements: Vec<Element>,
    time: f32,
}

impl OsuStoryboard {
    pub async fn new(
        def: StoryboardDef, 
        dir: String, 
        skin_manager: &mut SkinManager,
    ) -> TatakuResult<Self> {
        let settings = Settings::get().osu_settings.clone();
        let window_size = WindowSize::get();
        let scaling_helper = Arc::new(ScalingHelper::new_with_settings_custom_size(&settings, 0.0, window_size.0, false, GAME_SIZE));

        let mut image_cache = HashMap::new();
        let mut elements = Vec::new();
        for e in def.entries.clone() {
            elements.push(Element::new(e, &dir, &mut image_cache, &scaling_helper, skin_manager).await?);
        }
        elements.reverse();
        elements.sort_by(Element::sort);

        for i in elements.iter_mut() {
            i.window_size_changed(&scaling_helper);
        }

        Ok(Self {
            time: 0.0,
            scaling_helper,
            settings,
            elements
        })
    }

    // pub fn resize(&mut self, )
}


#[async_trait]
impl BeatmapAnimation for OsuStoryboard {
    async fn update(&mut self, time: f32, manager: &GameplayManager) {
        self.time = time;
        for i in self.elements.iter_mut() {
            // if self.time < i.start_time || self.time > i.end_time + 5000.0 { continue }
            i.update(time, manager);
        }
    }

    async fn draw(&self, list: &mut RenderableCollection) {
        for i in self.elements.iter() {
            // if self.time < i.start_time || !i.group.visible() { continue } // || (i.end_time < self.time && !i.group.visible()) { continue }
            // if !i.group.visible() { continue } // || (i.end_time < self.time && !i.group.visible()) { continue }

            let mut g = i.group.clone();

            g.pos.current = self.scaling_helper.scale_coords(g.pos.current);
            // g.scale.current *= self.scaling_helper.scale;

            list.push(g)
        }
    }

    fn window_size_changed(&mut self, size: Vector2) {
        self.scaling_helper = Arc::new(ScalingHelper::new_with_settings(&self.settings, 0.0, size, false));

        for i in self.elements.iter_mut() {
            i.window_size_changed(&self.scaling_helper)
        }
    }

    fn fit_to_area(&mut self, bounds: Bounds) {
        self.scaling_helper = Arc::new(ScalingHelper::new_offset_scale(
            5.0, 
            bounds.size, 
            bounds.pos, 
            0.5, 
            false,
        ));

        for i in self.elements.iter_mut() {
            i.window_size_changed(&self.scaling_helper)
        }
    }

    fn reset(&mut self) {
        for i in self.elements.iter_mut() {
            i.reset();
        }
    }
}

struct Element {
    start_time: f32,
    end_time: f32,
    layer: Layer,
    element_image: ElementImage,
    def: StoryboardElementDef,
    commands: Vec<StoryboardCommand>,
    // command_index: usize,
    group: TransformGroup,
}
impl Element {
    async fn new(
        def: StoryboardEntryDef, 
        parent_dir: &String, 
        image_cache: &mut HashMap<String, Image>, 
        scale: &ScalingHelper, 
        skin_manager: &mut SkinManager
    ) -> TatakuResult<Self> {
        let layer;

        let mut blend_mode = None;
        for i in def.commands.iter() {
            if let StoryboardEvent::Parameter { param: Param::AdditiveBlending } = i.event {
                blend_mode = Some(BlendMode::AdditiveBlending);
                break;
            }
        }

        
        let mut group = TransformGroup::new(Vector2::ZERO).border_alpha(0.0).alpha(0.0);
        let image = match def.element.clone() {
            StoryboardElementDef::Sprite(sprite) => {
                let filepath = format!("{parent_dir}/{}", sprite.filepath)
                    .replace("\\\\", "/")
                    .replace("\\", "/")
                ;

                let mut image = try_load_image(&filepath, image_cache, skin_manager).await?;
                // let mut image = if let Some(image) = image_cache.get(&filepath).cloned() {
                //     image
                // } else if let Some(i) = skin_manager.get_texture_noskin(&filepath, false).await {
                //     image_cache.insert(filepath, i.clone());
                //     i
                // } else {
                //     // try to find a file with the same name but different case
                //     let file_path = Path::new(&filepath).canonicalize().unwrap();
                //     let parent = file_path.parent().unwrap();

                //     let files = std::fs::read_dir(parent)?;
                //     let mut found = None;
                //     for file in files.filter_map(Result::ok) {
                //         if file.file_name().to_ascii_lowercase() != file_path.file_name().unwrap().to_ascii_lowercase() { continue }

                //         found = skin_manager.get_texture_noskin(&filepath, false).await;
                //         break;
                //     }

                //     let Some(image) = found else {
                //         return Err(TatakuError::String(format!("Image not found: {filepath}")))
                //     };

                //     image
                // };

                // apply origin
                image.origin = sprite.origin.resolve(image.tex_size());

                // let size = image.tex_size();
                // match sprite.origin {
                //     Origin::Custom => image.origin = Vector2::ZERO,

                //     Origin::TopCentre => image.origin.y = 0.0,
                //     Origin::TopLeft => image.origin = Vector2::ZERO,
                //     Origin::TopRight => image.origin = Vector2::new(size.x, 0.0),

                //     Origin::CentreLeft => image.origin.x = 0.0,
                //     Origin::Centre => image.origin = size / 2.0, // default
                //     Origin::CentreRight => image.origin.x = size.x,
                    
                //     Origin::BottomLeft => image.origin = Vector2::new(0.0, size.y),
                //     Origin::BottomCentre => image.origin.y = size.y,
                //     Origin::BottomRight => image.origin = size,
                // }

                layer = sprite.layer;
                blend_mode.map(|b| image.set_blend_mode(b));

                if sprite.filepath == "sb\\glow.png" {
                    image.draw_debug = true;
                }

                group.items.push(Arc::new(image.clone()));
                ElementImage::Sprite(image)
            }
            StoryboardElementDef::Animation(anim) => {
                let filepath = Path::new(&anim.filepath);
                let Some(ext) = filepath.extension() else { return Err(TatakuError::String("no extention on anim image".to_owned())); };
                let ext = ext.to_str().unwrap();
                let filename = filepath.to_str().unwrap().trim_end_matches(&format!(".{ext}"));

                // let Some(ext_ind) = anim.filepath.chars().enumerate().filter(|(_, c)| *c == '.').map(|(n, _)|n).last() else { return Err(TatakuError::String("no extention on anim image".to_owned())); };
                // let (filename, ext) = anim.filepath.split_at(ext_ind);

                let mut frames = Vec::new();
                let mut counter = 0;
                loop {
                    let filepath = format!("{parent_dir}/{filename}{counter}.{ext}")
                        .replace("\\\\", "/")
                        .replace("\\", "/")
                    ;

                    let Ok(image) = try_load_image(&filepath, image_cache, skin_manager).await else { 
                        if counter == 0 { error!("image not found: {filepath}"); }
                        break 
                    };

                    frames.push(image.tex);
                    counter += 1;
                }
                if frames.len() == 0 { return Err(TatakuError::String("anim has no frames!".to_owned())) }

                let delays = vec![anim.frame_delay; frames.len()];
                let tex_size = Vector2::new(frames[0].width as f32, frames[0].height as f32);
                let mut animation = Animation::new(Vector2::ZERO, Vector2::ONE, frames, delays, Vector2::ONE);
                animation.scale = Vector2::ONE;
                // animation.free_on_drop = true;
                animation.draw_debug = true;
                blend_mode.map(|b| animation.set_blend_mode(b));
                
                animation.origin = anim.origin.resolve(tex_size);

                layer = anim.layer;
                group.items.push(Arc::new(animation.clone()));
                ElementImage::Anim(animation)
            }
        };

        let mut s = Self {
            start_time: 0.0,
            end_time: 0.0,
            def: def.element,
            layer,
            element_image: image,
            commands: def.commands,
            // command_index: 0,
            group,
        };
        s.apply_commands(scale);

        Ok(s)
    }

    fn apply_commands(&mut self, scale: &ScalingHelper) {
        self.group.transforms.clear();

        let pos = match &self.def {
            StoryboardElementDef::Sprite(s) => s.pos,
            StoryboardElementDef::Animation(a) => a.pos,
        };
        self.group.pos.current = pos;

        // let origin = match &self.element_image {
        //     ElementImage::Sprite(i) => i.origin,
        //     ElementImage::Anim(a) => a.origin,
        // };

        // TODO: 
        // //if these are wrong, they will be updated next frame anyways
        // self.group.pos.both(scale.scale_coords(pos));
        // self.group.scale.both(Vector2::ONE * scale.scale);

        // match &mut self.element_image {
        //     ElementImage::Sprite(image) => {
        //         // image.scale = Vector2::ONE * scale.scale;
        //         image.pos = scale.scale_coords(pos);
        //         self.group.items = vec![Arc::new(image.clone())]
        //     }
        //     ElementImage::Anim(anim) => {
        //         anim.scale = Vector2::ONE * scale.scale;
        //         anim.pos = scale.scale_coords(pos);
        //         self.group.items = vec![Arc::new(anim.clone())]
        //     }
        // }

        let mut earliest_start:f32 = f32::MAX;
        let mut latest_end:f32 = 0.0;

        for i in self.commands.iter() {
            let offset = i.start_time;
            let mut duration = i.end_time - i.start_time;

            if duration < 0.0 {
                // error!("duration < 0.0: duration: {duration}, offset: {offset}, type: {trans_type:?}");
                // continue
                duration = duration.abs();
            }

            // duration = duration.max(500.0);

            earliest_start = earliest_start.min(i.start_time);
            latest_end = latest_end.max(i.end_time);


            let trans_type = match i.event {
                // // raw
                // StoryboardEvent::Move { start, end } => TransformType::Position { start, end },
                // StoryboardEvent::MoveX { start_x, end_x } => TransformType::PositionX { start: start_x, end: end_x },
                // StoryboardEvent::MoveY { start_y, end_y } => TransformType::PositionY { start: start_y, end: end_y },

                // scaling
                StoryboardEvent::Move { start, end } => TransformType::Position { 
                    start: scale.scale_coords(start), 
                    end:   scale.scale_coords(end)
                },
                StoryboardEvent::MoveX { start_x, end_x } => TransformType::PositionX { 
                    start: scale.scale_coords(Vector2::with_x(start_x)).x, 
                    end:   scale.scale_coords(Vector2::with_x(end_x)).x 
                },
                StoryboardEvent::MoveY { start_y, end_y } => TransformType::PositionY { 
                    start: scale.scale_coords(Vector2::with_y(start_y)).y, 
                    end:   scale.scale_coords(Vector2::with_y(end_y)).y 
                },

                // // scaling + origin offset 
                // StoryboardEvent::Move { start, end } => TransformType::Position { 
                //     start: scale.scale_coords(start - origin), 
                //     end:   scale.scale_coords(end - origin)
                // },
                // StoryboardEvent::MoveX { start_x, end_x } => TransformType::PositionX { 
                //     start: scale.scale_coords(Vector2::with_x(start_x) - origin).x, 
                //     end:   scale.scale_coords(Vector2::with_x(end_x) - origin).x 
                // },
                // StoryboardEvent::MoveY { start_y, end_y } => TransformType::PositionY { 
                //     start: scale.scale_coords(Vector2::with_y(start_y) - origin).y, 
                //     end:   scale.scale_coords(Vector2::with_y(end_y) - origin).y 
                // },


                // StoryboardEvent::Scale { start_scale, end_scale } => TransformType::Scale { start: start_scale, end: end_scale },
                // StoryboardEvent::VectorScale { start_scale, end_scale } => TransformType::VectorScale { start: start_scale, end: end_scale },
                StoryboardEvent::Scale { start_scale, end_scale } => TransformType::Scale { start: start_scale * scale.scale, end: end_scale * scale.scale },
                StoryboardEvent::VectorScale { start_scale, end_scale } => TransformType::VectorScale { start: start_scale * scale.scale, end: end_scale * scale.scale },

                StoryboardEvent::Fade { start, end } => TransformType::Transparency { start, end },

                StoryboardEvent::Rotate { start_rotation, end_rotation } => TransformType::Rotation { start: start_rotation, end: end_rotation },
                StoryboardEvent::Color { start_color, end_color } => TransformType::Color { start: start_color, end: end_color },
                StoryboardEvent::Parameter { param } => match param {
                    Param::FlipHorizontal => { self.group.image_flip_horizonal = true; continue; },
                    Param::FlipVertial => { self.group.image_flip_vertical = true; continue; },
                    _ => continue
                } 
                StoryboardEvent::Loop { loop_count:_ } => continue,

                _ => continue
            };

            self.group.transforms.push(Transformation::new(
                offset, 
                duration,
                trans_type,
                i.easing.into(),
                0.0
            ));
        }

        self.start_time = earliest_start;
        self.end_time = latest_end;
    }

    fn update(&mut self, time: f32, _manager: &GameplayManager) {
        // if time < self.start_time || time > self.end_time { 
        //     self.group.update(time as f64);
        //     return 
        // }

        if let ElementImage::Anim(anim) = &mut self.element_image {
            let old_frame = anim.frame_index;
            anim.update(time);

            if anim.frame_index != old_frame {
                self.group.items = vec![Arc::new(anim.clone())]
            }
        }

        self.group.update(time)
    }

    fn window_size_changed(&mut self, scale: &Arc<ScalingHelper>) {
        self.apply_commands(scale);
    }

    fn reset(&mut self) {
        if let ElementImage::Anim(anim) = &mut self.element_image {
            anim.update(0.0);

            self.group.items = vec![Arc::new(anim.clone())]
        }

        self.group.update(0.0)
    }

    fn sort(a: &Self, b: &Self) -> std::cmp::Ordering {
        // let size = match & a.element_image {
        //     ElementImage::Sprite(s) => s.size(),
        //     ElementImage::Anim(a) => a.size(),
        // };

        // if size > 

        // b.layer.cmp(&a.layer) // should be correct // was not correct
        a.layer.cmp(&b.layer)
    }

}

// impl Drop for Element {
//     fn drop(&mut self) {
//         match &self.element_image {
//             ElementImage::Sprite(i) => {
//                 GameWindow::free_texture(i.tex);
//             }
//             ElementImage::Anim(a) => {
//                 for i in &a.frames {
//                     GameWindow::free_texture(*i);
//                 }
//             },
//         }
//     }
// }


enum ElementImage {
    Sprite(Image),
    Anim(Animation),
}


async fn try_load_image(
    filepath: &String,
    image_cache: &mut HashMap<String, Image>, 
    skin_manager: &mut SkinManager
) -> TatakuResult<Image> {
    if let Some(image) = image_cache.get(filepath).cloned() {
        Ok(image)
    } else if let Some(i) = skin_manager.get_texture_noskin(filepath, false).await {
        image_cache.insert(filepath.clone(), i.clone());
        Ok(i)
    } else {
        // try to find a file with the same name but different case
        let file_path = Path::new(&filepath);
        let parent = file_path.parent().unwrap();
        let filename = file_path.file_name().unwrap().to_ascii_lowercase();

        let files = std::fs::read_dir(parent)?;
        let mut found = None;
        for file in files.filter_map(Result::ok) {
            if file.file_name().to_ascii_lowercase() != filename { continue }
            // let filename = file.file_name().to_str().unwrap();
            let filepath2 = parent.join(file.file_name()).to_string_lossy().to_string();
            found = skin_manager.get_texture_noskin(&filepath2, false).await;
            if found.is_some() {
                warn!("using file {filepath2} instead of {filepath} for storyboard");
            }

            break;
        }

        let Some(image) = found else {
            return Err(TatakuError::String(format!("Image not found: {filepath}")))
        };

        Ok(image)
    }
}

// /// peppy fns
// fn easein_back<T:Interpolatable>(current:T, target: T, amount: f64) -> T {
//     if amount == 0.0 {
//         current
//     } else if amount == 1.0 {
//         target
//     } else {
//         let s = 1.70158;
//         let change = target - current;

//         current + change * amount * ((s + 1.0) * amount - s)
//     }
// }
// fn easeout_back<T:Interpolatable>(current:T, target: T, amount: f64) -> T {
//     if amount == 0.0 {
//         current
//     } else if amount == 1.0 {
//         target
//     } else {
//         let s = 1.70158;
//         let change = target - current;
//         current + change * ((amount - 1.0) * amount * ((s + 1.0) * amount + s) + 1.0)
//         // return current + change * ((amount - 1) * amount * ((s + 1) * amount + s) + 1);
//     }
// }
// fn easeinout_back<T:Interpolatable>(current:T, target: T, amount: f64) -> T {
//     if amount == 0.0 {
//         current
//     } else if amount == 1.0 {
//         target
//     } else {
//         let s = 1.70158* 1.525;
//         let change = target - current;
        
//         // i dont know how this is supposed to happen since amount should generally be between 0.0 and 1.0
//         if (amount / 2.0) < 1.0 { 
//             current + change / 2.0 * (amount.powi(2) * ((s + 1.0) * amount - s))
//         } else {
//             let amount = amount - 2.0;
//             current + change / 2.0 * (amount.powi(2) * ((s + 1.0) * amount + s) + 2.0)
//         }
//     }
// }


// fn easein_bounce<T:Interpolatable>(current:T, target: T, amount: f64) -> T {
//     if amount == 0.0 {
//         current
//     } else if amount == 1.0 {
//         target
//     } else {
//         let change = target - current;
//         // ApplyEasing(EasingTypes.OutBounce, duration - time, 0, change, duration) + initial;
//         current + easeout_bounce(0.0, target - current, amount)
//     }
// }
// fn easeout_bounce<T:Interpolatable>(current:T, target: T, amount: f64) -> T {
//     if amount == 0.0 {
//         current
//     } else if amount == 1.0 {
//         target
//     } else {
//         // if ((time /= duration) < 1 / 2.75)
//         //     return change * (7.5625 * time * time) + initial;
//         // else if (time < 2 / 2.75)
//         //     return change * (7.5625 * (time -= 1.5 / 2.75) * time + .75) + initial;
//         // else if (time < 2.5 / 2.75)
//         //     return change * (7.5625 * (time -= 2.25 / 2.75) * time + .9375) + initial;
//         // else
//         //     return change * (7.5625 * (time -= 2.625 / 2.75) * time + .984375) + initial;
//         let time = amount;
//         let change = target - current;

//         if (amount < 1.0 / 2.75){
//             current + change * (7.5625 * time * time)
//         } else if (time < 2.0 / 2.75) {
//             current + change * (7.5625 * (time -= 1.5 / 2.75) * time + 0.75)
//         } else if (time < 2.5 / 2.75) {
//             current + change * (7.5625 * (time -= 2.25 / 2.75) * time + 0.9375)
//         } else {
//             current + change * (7.5625 * (time -= 2.625 / 2.75) * time + 0.984375)
//         }
//     }
// }