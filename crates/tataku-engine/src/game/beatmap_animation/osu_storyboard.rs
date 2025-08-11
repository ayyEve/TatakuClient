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
    pub fn new(
        def: &StoryboardDef,
        dir: &String,
        skin_manager: &mut dyn SkinProvider,
        // settings: OsuSettings,
    ) -> TatakuResult<Self> {
        let playfield_size = GAME_SIZE;

        let transform = Transform::default();
        let playfield = Bounds::new(
            Vector2::ZERO,
            playfield_size
        );

        let mut image_cache = HashMap::new();
        let mut elements = Vec::new();
        for e in def.entries.clone() {
            elements.push(Element::new(e, dir, &mut image_cache, skin_manager)?);
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
impl BeatmapAnimation for OsuStoryboard {
    fn use_gamemode_playfield(&self, gamemode: &GamemodeInfo) -> bool {
        gamemode.id == "osu"
    }


    fn update(&mut self, time: f32) {
        self.time = time;
        for i in self.elements.iter_mut() {
            // if self.time < i.start_time || self.time > i.end_time + 5000.0 { continue }
            i.update(time);
        }
    }

    fn draw(&self, list: &mut RenderableCollection) {
        // list.push_scissor(self.bounds.into_scissor());
        let bounds = self.playfield;
        let scissor = bounds.into_scissor();

        for i in self.elements.iter() {
            if !i.visible(self.time) { continue }

            let image_flip = ImageFlip::new(
                i.flip_horizontal.last_value() != 0.0,
                i.flip_vertical.last_value() != 0.0
            );

            let draw_options = DrawOptions {
                image_flip,
                ..Default::default()
            };

            let x_position = i.x_position.last_value();
            let y_position = i.y_position.last_value();
            let rotation = i.rotation.last_value();
            let x_scale = i.x_scale.last_value();
            let y_scale = i.y_scale.last_value();

            let transform = Transform::new(
                Vector2::new(x_position, y_position),
                Vector2::new(x_scale, y_scale),
                rotation,
                i.origin
            );

            let transform = transform.translate(-self.transform.origin)
                .rotate(self.transform.rotation)
                .scale(self.transform.scale)
                .translate(self.transform.pos);

            let alpha = i.alpha.last_value();
            let color = i.color.last_value().alpha(alpha);

            let element: Box<dyn TatakuRenderable> = match i.element_image.clone() {
                ElementImage::Sprite(mut image) => {
                    image.color = color;

                    Box::new(image)
                },
                ElementImage::Anim(mut animation) => {
                    animation.color = color;

                    Box::new(animation)
                },
            };

            list.push(Scissored::new(
                scissor,
                Box::new(MergeDrawOptions::new(
                    draw_options,
                    Box::new(Transformed::new(
                        transform,
                        element
                    ))
                ))
            ));
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

    initial_pos: Vector2,
    origin: Vector2,
    x_position: AnimationTimeline<f32>,
    y_position: AnimationTimeline<f32>,
    rotation: AnimationTimeline<f32>,
    x_scale: AnimationTimeline<f32>,
    y_scale: AnimationTimeline<f32>,

    // todo: do properly
    flip_horizontal: AnimationTimeline<f32>,
    flip_vertical: AnimationTimeline<f32>,

    alpha: AnimationTimeline<f32>,
    color: AnimationTimeline<Color>,
}
impl Element {
    fn new(
        def: StoryboardEntryDef,
        parent_dir: &String,
        image_cache: &mut HashMap<String, Image>,
        skin_manager: &mut dyn SkinProvider
    ) -> TatakuResult<Self> {
        let layer;

        let mut blend_mode = None;
        for i in def.commands.iter() {
            let StoryboardEvent::Parameter { 
                param: Param::AdditiveBlending 
            } = i.event else { continue };
            // if i.start_time as i32 == i.end_time as i32 {
                blend_mode = Some(Pipeline::OsuAdditiveBlending);
            // }
            break;
        }

        let initial_pos;
        let origin;

        let image = match def.element.clone() {
            StoryboardElementDef::Sprite(sprite) => {
                let filepath = format!("{parent_dir}/{}", sprite.filepath)
                    .replace("\\\\", "/")
                    .replace("\\", "/")
                ;

                let mut image = try_load_image(
                    &filepath, 
                    image_cache, 
                    skin_manager
                )?;

                image.origin = Vector2::ZERO;
                image.pos = Vector2::ZERO;

                layer = sprite.layer;
                initial_pos = sprite.pos;
                origin = sprite.origin.resolve(image.tex_size());

                if let Some(b) = blend_mode { image.set_blend_mode(b); }

                ElementImage::Sprite(image)
            }
            StoryboardElementDef::Animation(anim) => {
                let filepath = Path::new(&anim.filepath);
                let Some(ext) = filepath.extension() else { 
                    return Err(TatakuError::String("no extention on anim image".to_owned())); 
                };

                let ext = ext.to_str().unwrap();
                let filename = filepath.to_str().unwrap().trim_end_matches(&format!(".{ext}"));

                let mut frames = Vec::new();
                let mut counter = 0;
                loop {
                    let filepath = format!("{parent_dir}/{filename}{counter}.{ext}")
                        .replace("\\\\", "/")
                        .replace("\\", "/")
                    ;

                    let Ok(image) = try_load_image(
                        &filepath, 
                        image_cache, 
                        skin_manager
                    ) else {
                        if counter == 0 { error!("image not found: {filepath}"); }
                        break
                    };

                    frames.push(image.tex);
                    counter += 1;
                }
                if frames.is_empty() { 
                    return Err(TatakuError::String("anim has no frames!".to_owned())) 
                }

                let tex_size = Vector2::new(
                    frames[0].width as f32, 
                    frames[0].height as f32
                );

                let mut animation = Animation::new(
                    Vector2::ZERO, 
                    Vector2::ONE, 
                    frames, 
                    anim.frame_delay, 
                    Vector2::ONE
                );
                animation.origin = Vector2::ZERO;
                animation.scale = Vector2::ONE;
                animation.draw_debug = true;
                if let Some(b) = blend_mode { animation.set_blend_mode(b); }

                initial_pos = anim.pos;
                origin = anim.origin.resolve(tex_size);
                layer = anim.layer;

                ElementImage::Anim(animation)
            }
        };

        let mut s = Self {
            start_time: 0.0,
            end_time: 0.0,
            layer,
            element_image: image,
            commands: def.commands,

            initial_pos,
            origin,
            x_position: AnimationTimeline::new(Vec::new(), initial_pos.x),
            y_position: AnimationTimeline::new(Vec::new(), initial_pos.y),
            rotation: AnimationTimeline::new(Vec::new(), 0.0),
            x_scale: AnimationTimeline::new(Vec::new(), 1.0),
            y_scale: AnimationTimeline::new(Vec::new(), 1.0),

            flip_horizontal: AnimationTimeline::new(Vec::new(), 0.0),
            flip_vertical: AnimationTimeline::new(Vec::new(), 0.0),

            alpha: AnimationTimeline::new(Vec::new(), 1.0),
            color: AnimationTimeline::new(Vec::new(), Color::WHITE),
        };
        s.apply_commands();

        Ok(s)
    }

    fn apply_commands(&mut self) {
        let mut x_position = Vec::new();
        let mut y_position = Vec::new();
        let mut rotation = Vec::new();
        let mut x_scale = Vec::new();
        let mut y_scale = Vec::new();

        let mut flip_horizontal: Vec<Animate<f32>> = Vec::new();
        let mut flip_vertical: Vec<Animate<f32>> = Vec::new();

        let mut alpha = Vec::new();
        let mut color = Vec::new();

        let mut earliest_start = f32::MAX;
        let mut latest_end = 0.0f32;

        for i in self.commands.iter() {
            let mut duration = i.end_time - i.start_time;

            // i wonder if durations that are less than 0 should be run immediately?
            if duration < 0.0 {
                // warn!("duration < 0.0: command: {i:?}");
                duration = duration.abs();
            }

            // duration = duration.max(500.0);

            earliest_start = earliest_start.min(i.start_time);
            latest_end = latest_end.max(i.end_time);


            match i.event {
                StoryboardEvent::Move { start, end } => {
                    x_position.push(Animate::new(i.start_time, duration, i.easing.into(), start.x, end.x));
                    y_position.push(Animate::new(i.start_time, duration, i.easing.into(), start.y, end.y));
                },
                StoryboardEvent::MoveX { start, end } =>
                    x_position.push(Animate::new(i.start_time, duration, i.easing.into(), start, end)),
                StoryboardEvent::MoveY { start, end } =>
                    y_position.push(Animate::new(i.start_time, duration, i.easing.into(), start, end)),

                StoryboardEvent::Scale { start, end } => {
                    x_scale.push(Animate::new(i.start_time, duration, i.easing.into(), start, end));
                    y_scale.push(Animate::new(i.start_time, duration, i.easing.into(), start, end));
                },
                StoryboardEvent::VectorScale { start, end } => {
                    x_scale.push(Animate::new(i.start_time, duration, i.easing.into(), start.x, end.x));
                    y_scale.push(Animate::new(i.start_time, duration, i.easing.into(), start.y, end.y));
                },

                StoryboardEvent::Rotate { start, end } =>
                    rotation.push(Animate::new(i.start_time, duration, i.easing.into(), start, end)),

                StoryboardEvent::Fade { start, end } =>
                    alpha.push(Animate::new(i.start_time, duration, i.easing.into(), start, end)),

                StoryboardEvent::Color { start, end } =>
                    color.push(Animate::new(i.start_time, duration, i.easing.into(), start, end)),

                StoryboardEvent::Parameter { param } => match param {
                    Param::FlipHorizontal => {
                        let start = flip_horizontal.last().map(|animation| animation.end).unwrap_or_default();

                        flip_horizontal.push(Animate::new(i.start_time, duration, Easing::default(), start, 1.0));
                        flip_horizontal.push(Animate::new(i.end_time, 0.0, Easing::default(), 1.0, 0.0));
                    },
                    Param::FlipVertial => {
                        let start = flip_vertical.last().map(|animation| animation.end).unwrap_or_default();

                        flip_vertical.push(Animate::new(i.start_time, duration, Easing::default(), start, 1.0));
                        flip_vertical.push(Animate::new(i.end_time, 0.0, Easing::default(), 1.0, 0.0));
                    },
                    Param::AdditiveBlending => {} // todo:
                }
                StoryboardEvent::Loop { .. } => {}, // done elsewhere
            };
        }

        self.x_position = AnimationTimeline::new(x_position, self.initial_pos.x);
        self.y_position = AnimationTimeline::new(y_position, self.initial_pos.y);
        self.rotation = AnimationTimeline::new(rotation, 0.0);
        self.x_scale = AnimationTimeline::new(x_scale, 1.0);
        self.y_scale = AnimationTimeline::new(y_scale, 1.0);

        self.flip_horizontal = AnimationTimeline::new(flip_horizontal, 0.0);
        self.flip_vertical = AnimationTimeline::new(flip_vertical, 0.0);

        self.alpha = AnimationTimeline::new(alpha, 1.0);
        self.color = AnimationTimeline::new(color, Color::WHITE);


        self.start_time = earliest_start;
        self.end_time = latest_end;
    }

    fn visible(&self, time: f32) -> bool {
        time >= self.start_time && time < self.end_time
            && self.alpha.last_value() != 0.0
            && self.x_scale.last_value() != 0.0 && self.y_scale.last_value() != 0.0
    }

    fn update(&mut self, time: f32) {
        if let ElementImage::Anim(anim) = &mut self.element_image {
            anim.update(time);
        }

        self.x_position.update(time);
        self.y_position.update(time);
        self.rotation.update(time);
        self.x_scale.update(time);
        self.y_scale.update(time);

        self.flip_horizontal.update(time);
        self.flip_vertical.update(time);

        self.alpha.update(time);
        self.color.update(time);
    }

    fn reset(&mut self) {
        if let ElementImage::Anim(anim) = &mut self.element_image {
            anim.update(0.0);
        }
    }

    fn sort(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.layer.cmp(&b.layer)
    }

}

#[derive(Clone)]
enum ElementImage {
    Sprite(Image),
    Anim(Animation),
}

fn try_load_image(
    filepath: &String,
    image_cache: &mut HashMap<String, Image>,
    skin_manager: &mut dyn SkinProvider
) -> TatakuResult<Image> {
    if let Some(image) = image_cache.get(filepath).cloned() {
        Ok(image)
    } else if let Some(i) = skin_manager.get_texture(
        filepath, 
        &TextureSource::Raw, 
        SkinUsage::Beatmap, 
        false
    ) {
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
            let filepath2 = parent
                .join(file.file_name())
                .to_string_lossy()
                .to_string();
            found = skin_manager.get_texture(
                &filepath2, 
                &TextureSource::Raw, 
                SkinUsage::Beatmap, 
                false
            );
            break;
        }

        let Some(image) = found else {
            return Err(TatakuError::String(format!("Image not found: {filepath}")))
        };

        Ok(image)
    }
}
