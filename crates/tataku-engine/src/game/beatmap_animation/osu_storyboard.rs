use crate::prelude::*;

const GAME_SIZE: Vector2 = Vector2::new(640.0, 480.0);
const OFFSET: Vector2 = Vector2::new(64.0, 56.0);

pub struct OsuStoryboard {
    // scaling_helper: Arc<ScalingHelper>,
    playfield_size: Vector2,
    transform: Transform,
    playfield: Bounds,

    settings: OsuSettings,

    elements: Vec<Element>,
    time: f32,
}

impl OsuStoryboard {
    pub async fn new(
        def: StoryboardDef,
        dir: String,
        skin_manager: &mut dyn SkinProvider,
        settings: OsuSettings,
    ) -> TatakuResult<Self> {
        let playfield_size = GAME_SIZE;

        let transform = Transform::default();
        let playfield = Bounds::new(
            transform.matrix() * Vector2::ZERO,
            transform.matrix() * playfield_size
        );

        // let window_size = WindowSize::get();
        // let scaling_helper = Arc::new(ScalingHelper::new_with_settings_custom_size(&settings, 0.0, window_size.0, false, GAME_SIZE));

        let mut image_cache = HashMap::new();
        let mut elements = Vec::new();
        for e in def.entries.clone() {
            elements.push(Element::new(e, &dir, &mut image_cache,  skin_manager).await?);
        }
        // elements.reverse();
        elements.sort_by(Element::sort);

        for i in elements.iter_mut() {
            i.apply_commands();
        }

        Ok(Self {
            time: 0.0,
            settings,
            elements,
            playfield_size,

            transform,
            playfield,
        })
    }

    // pub fn resize(&mut self, )
}


#[async_trait]
impl BeatmapAnimation for OsuStoryboard {
    fn use_gamemode_playfield(&self, gamemode: &GameModeInfo) -> bool {
        gamemode.id == "osu"
    }


    async fn update(&mut self, time: f32) {
        self.time = time;
        for i in self.elements.iter_mut() {
            // if self.time < i.start_time || self.time > i.end_time + 5000.0 { continue }
            i.update(time);
        }
    }

    async fn draw(&self, list: &mut RenderableCollection) {
        // list.push_scissor(self.bounds.into_scissor());
        let bounds = self.playfield;

        let scissor = bounds.into_scissor();

        for i in self.elements.iter() {
            if self.time < i.start_time || !i.group.visible() { continue } // || (i.end_time < self.time && !i.group.visible()) { continue }
            // if !i.group.visible() { continue } // || (i.end_time < self.time && !i.group.visible()) { continue }
            let mut group = i.group.clone();
            group.scissor = Some(scissor);
            list.push(TransformedDrawable::new(self.transform, Box::new(group)));
        }

        list.push(Rectangle::new_bounds(
            bounds,
            Color::TRANSPARENT_WHITE,
            Some(Border::new(Color::GREEN, 2.0))
        ));

        // list.pop_scissor();
    }

    fn window_size_changed(&mut self, size: Vector2) {
        // debug!("window size: {size}");
        let nonsense = PlayfieldNonsense::new(
            Bounds::new(Vector2::ZERO, size),
            1.0,
            Vector2::ZERO,
            false
        );
        self.fit_to_area(nonsense);


        // let (scale, pos) = self.settings.get_playfield();

        // self.transform = ScalingHelper::new_transform(
        //     size,
        //     pos,
        //     scale,
        //     false,
        //     None
        // );

        // self.playfield = Bounds::new(
        //     self.transform.matrix() * Vector2::ZERO,
        //     self.transform.matrix() * self.playfield_size
        // );
    }

    fn fit_to_area(&mut self, nonsense: PlayfieldNonsense) {
        // self.scaling_helper = Arc::new(ScalingHelper::new_offset_scale(
        //     5.0,
        //     bounds.size,
        //     bounds.pos,
        //     0.5,
        //     false,
        // ));

        let transform = Transform::new(
            nonsense.bounds.pos + if nonsense.flip_vertical { Vector2::new(0.0, nonsense.bounds.size.y) } else { Vector2::ZERO },
            Vector2::new(1.0, if nonsense.flip_vertical { -1.0 } else { 1.0 }) * nonsense.scale,
            0.0,
            OFFSET,
        );

        self.transform = transform;

        // self.transform = ScalingHelper::transform_padded(
        //     transform,
        //     nonsense.circle_size.x,
        //     None
        // );

        let tl = transform.matrix() * Vector2::ZERO;
        let br = transform.matrix() * self.playfield_size;

        self.playfield = Bounds::new(
            tl,
            br - tl
        );

        // debug!("{nonsense:#?}, playfield: {:?}", self.playfield_size);
        // debug!("transform: {transform:?}, bounds: {:?}", self.playfield);

        // self.scaling_helper = Arc::new(ScalingHelper::fit_to_playfield(
        //     nonsense,
        //     false,
        // ));
    }

    fn reset(&mut self) {
        self.elements.iter_mut().for_each(Element::reset);
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
        skin_manager: &mut dyn SkinProvider
    ) -> TatakuResult<Self> {
        let layer;

        let mut blend_mode = None;
        for i in def.commands.iter() {
            let StoryboardEvent::Parameter { param: Param::AdditiveBlending } = i.event else { continue };
            // if i.start_time as i32 == i.end_time as i32 {
                blend_mode = Some(BlendMode::OsuAdditiveBlending);
            // }
            break;
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

                image.origin = Vector2::ZERO;
                image.pos = Vector2::ZERO;

                group.pos = sprite.pos;
                group.origin = sprite.origin.resolve(image.tex_size());

                layer = sprite.layer;
                if let Some(b) = blend_mode { image.set_blend_mode(b) }

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
                if frames.is_empty() { return Err(TatakuError::String("anim has no frames!".to_owned())) }

                let delays = vec![anim.frame_delay; frames.len()];
                let tex_size = Vector2::new(frames[0].width as f32, frames[0].height as f32);
                let mut animation = Animation::new(Vector2::ZERO, Vector2::ONE, frames, delays, Vector2::ONE);
                animation.origin = Vector2::ZERO;
                animation.scale = Vector2::ONE;
                animation.draw_debug = true;
                if let Some(b) = blend_mode { animation.set_blend_mode(b) }


                group.pos = anim.pos;
                group.origin = anim.origin.resolve(tex_size);
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
        s.apply_commands();

        Ok(s)
    }

    fn apply_commands(&mut self) {
        self.group.transforms.clear();

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
                // scaling
                StoryboardEvent::Move { start, end } => TransformType::Position { start, end },
                StoryboardEvent::MoveX { start, end } => TransformType::PositionX { start, end },
                StoryboardEvent::MoveY { start, end } => TransformType::PositionY { start, end },

                StoryboardEvent::Scale { start, end } => TransformType::Scale { start, end },
                StoryboardEvent::VectorScale { start, end } => TransformType::VectorScale { start, end },

                StoryboardEvent::Fade { start, end } => TransformType::Transparency { start, end },

                StoryboardEvent::Rotate { start, end } => TransformType::Rotation { start, end },
                StoryboardEvent::Color { start, end } => TransformType::Color { start, end },

                StoryboardEvent::Parameter { param } => match param {
                    Param::FlipHorizontal => { self.group.image_flip_horizonal = true; continue; },
                    Param::FlipVertial => { self.group.image_flip_vertical = true; continue; },
                    _ => continue
                }
                StoryboardEvent::Loop { count:_ } => continue,

                // _ => continue
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

    fn update(&mut self, time: f32) {
        // if time < self.start_time || time > self.end_time {
        //     self.group.update(time as f64);
        //     return
        // }

        if let ElementImage::Anim(anim) = &mut self.element_image {
            let old_frame = anim.frame_index;
            anim.update(time);

            if anim.frame_index != old_frame {
                // only
                self.group.items = vec![Arc::new(anim.current_frame_as_image())]
            }
        }

        self.group.update(time)
    }

    // fn playfield_changed(&mut self) {
    //     self.apply_commands();
    // }

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

enum ElementImage {
    Sprite(Image),
    Anim(Animation),
}
impl ElementImage {
    fn size(&self) -> Vector2 {
        match self {
            Self::Sprite(s) => s.size(),
            Self::Anim(a) => a.size(),
        }
    }
}

async fn try_load_image(
    filepath: &String,
    image_cache: &mut HashMap<String, Image>,
    skin_manager: &mut dyn SkinProvider
) -> TatakuResult<Image> {
    if let Some(image) = image_cache.get(filepath).cloned() {
        Ok(image)
    } else if let Some(i) = skin_manager.get_texture(filepath, &TextureSource::Raw, SkinUsage::Beatmap, false).await {
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
            found = skin_manager.get_texture(&filepath2, &TextureSource::Raw, SkinUsage::Beatmap, false).await;
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