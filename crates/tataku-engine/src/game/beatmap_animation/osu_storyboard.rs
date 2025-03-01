use crate::prelude::*;

const GAME_SIZE: Vector2 = Vector2::new(640.0, 480.0);
const OFFSET: Vector2 = Vector2::new(64.0, 56.0);

pub struct OsuStoryboard {
    playfield_size: Vector2,
    transform: Transform,
    playfield: Bounds,

    elements: Vec<Element>,
    time: f32,
}
impl OsuStoryboard {
    pub async fn new(
        def: StoryboardDef,
        dir: String,
        skin_manager: &mut dyn SkinProvider,
        // settings: OsuSettings,
    ) -> TatakuResult<Self> {
        let playfield_size = GAME_SIZE;

        let transform = Transform::default();
        let playfield = Bounds::new(
            transform.matrix() * Vector2::ZERO,
            transform.matrix() * playfield_size
        );

        let mut image_cache = HashMap::new();
        let mut elements = Vec::new();
        for e in def.entries.clone() {
            elements.push(Element::new(e, &dir, &mut image_cache,  skin_manager).await?);
        }
        elements.sort_by(Element::sort);

        for i in elements.iter_mut() {
            i.apply_commands();
        }

        Ok(Self {
            time: 0.0,
            elements,
            playfield_size,

            transform,
            playfield,
        })
    }
}


#[async_trait]
impl BeatmapAnimation for OsuStoryboard {
    fn use_gamemode_playfield(&self, gamemode: &GamemodeInfo) -> bool {
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
        // list.pop_scissor();
    }

    fn window_size_changed(&mut self, size: Vector2) {
        let nonsense = PlayfieldNonsense::new(
            Bounds::new(Vector2::ZERO, size),
            1.0,
            Vector2::ZERO,
            false
        )
        .is_fullscreen(true);
        self.fit_to_area(nonsense);
    }

    fn fit_to_area(&mut self, mut nonsense: PlayfieldNonsense) {
        if nonsense.is_fullscreen {
            // debug!("window size: {size}");
            nonsense.scale = (nonsense.bounds.size / GAME_SIZE).min_component() * 0.90;

            nonsense.bounds.pos = Alignment::CENTER.resolve(
                &Bounds::new(
                    nonsense.bounds.pos + OFFSET * nonsense.scale,
                    nonsense.bounds.size
                ), 
                GAME_SIZE * nonsense.scale, 
                true, 
                true,
            );
        }

        let transform = Transform::new(
            nonsense.bounds.pos + if nonsense.flip_vertical { Vector2::new(0.0, nonsense.bounds.size.y) } else { Vector2::ZERO },
            Vector2::new(1.0, if nonsense.flip_vertical { -1.0 } else { 1.0 }) * nonsense.scale,
            0.0,
            OFFSET,
        );

        self.transform = transform;
        let tl = transform.matrix() * Vector2::ZERO;
        let br = transform.matrix() * self.playfield_size;

        self.playfield = Bounds::new(
            tl,
            br - tl
        );
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

            // i wonder if durations that are less than 0 should be run immediately?
            if duration < 0.0 {
                warn!("duration < 0.0: command: {i:?}");
                duration = duration.abs();
            }

            // duration = duration.max(500.0);

            earliest_start = earliest_start.min(i.start_time);
            latest_end = latest_end.max(i.end_time);


            let trans_type = match i.event {
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

    fn reset(&mut self) {
        if let ElementImage::Anim(anim) = &mut self.element_image {
            anim.update(0.0);

            self.group.items = vec![Arc::new(anim.clone())]
        }

        self.group.update(0.0)
    }

    fn sort(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.layer.cmp(&b.layer)
    }

}

enum ElementImage {
    #[allow(dead_code)] // this (probably?) holds a reference to the image so its not dropped and cleared
    Sprite(Image),
    Anim(Animation),
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
            // if found.is_some() {
            //     warn!("using file {filepath2} instead of {filepath} for storyboard");
            // }

            break;
        }

        let Some(image) = found else {
            return Err(TatakuError::String(format!("Image not found: {filepath}")))
        };

        Ok(image)
    }
}
