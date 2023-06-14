use crate::prelude::*;

const GAME_SIZE: Vector2 = Vector2::new(640.0, 480.0);
const DEPTH: Range<f32> = 5000.0..6000.0;

pub struct OsuStoryboard {
    scaling_helper: Arc<ScalingHelper>,
    settings: StandardSettings,

    elements: Vec<Element>,
    time: f32,
}

impl OsuStoryboard {
    pub async fn new(def: StoryboardDef, dir: String) -> TatakuResult<Self> {
        let settings = get_settings!().standard_settings.clone();
        let window_size = WindowSize::get();
        let scaling_helper = Arc::new(ScalingHelper::new_with_settings_custom_size(&settings, 0.0, window_size.0, false, GAME_SIZE));

        let mut image_cache = HashMap::new();
        let mut elements = Vec::new();
        for e in def.entries.clone() {
            elements.push(Element::new(e, &dir, &mut image_cache, &scaling_helper).await?);
        }
        elements.sort_by(Element::sort);

        let len = elements.len() as f32;
        for (n, i) in elements.iter_mut().enumerate() {
            let d = f32::lerp(DEPTH.start, DEPTH.end, (n as f32) / len);
            i.group.depth = d;
            i.window_size_changed(&scaling_helper);
        }

        Ok(Self {
            time: 0.0,
            scaling_helper,
            settings,
            elements
        })
    }
}


#[async_trait]
impl BeatmapAnimation for OsuStoryboard {
    async fn update(&mut self, time: f32, manager: &IngameManager) {
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
    async fn new(def: StoryboardEntryDef, parent_dir: &String, image_cache: &mut HashMap<String, Image>, scale: &ScalingHelper) -> TatakuResult<Self> {
        let layer;

        let mut group = TransformGroup::new(Vector2::ZERO, 0.0).border_alpha(0.0).alpha(0.0);
        let image = match def.element.clone() {
            StoryboardElementDef::Sprite(sprite) => {
                let filepath = format!("{parent_dir}/{}", sprite.filepath);
                let mut image = if let Some(image) = image_cache.get(&filepath).cloned() {
                    image
                } else if let Some(i) = load_image(&filepath, false, Vector2::ONE).await {
                    image_cache.insert(filepath, i.clone());
                    i
                } else {
                    return Err(TatakuError::String("Image not found: {filepath}".to_owned()))
                };

                // apply origin
                let size = image.tex_size();
                match sprite.origin {
                    Origin::Custom => image.origin = Vector2::ZERO,

                    Origin::TopCentre => image.origin.y = 0.0,
                    Origin::TopLeft => image.origin = Vector2::ZERO,
                    Origin::TopRight => image.origin = Vector2::new(size.x, 0.0),

                    Origin::CentreLeft => image.origin.x = 0.0,
                    Origin::Centre => image.origin = size / 2.0, // default
                    Origin::CentreRight => image.origin.x = size.x,
                    
                    Origin::BottomLeft => image.origin = Vector2::new(0.0, size.y),
                    Origin::BottomCentre => image.origin.y = size.y,
                    Origin::BottomRight => image.origin = size,
                }

                layer = sprite.layer;

                group.items.push(Arc::new(image.clone()));
                ElementImage::Sprite(image)
            }
            StoryboardElementDef::Animation(anim) => {
                let Some(ext_ind) = anim.filepath.chars().enumerate().filter(|(_, c)|*c == '.').map(|(n, _)|n).last() else { return Err(TatakuError::String("no extention on anim image".to_owned())); };
                let (filename, ext) = anim.filepath.split_at(ext_ind);

                let mut frames = Vec::new();
                let mut counter = 0;
                loop {
                    let filepath = format!("{parent_dir}/{filename}{counter}.{ext}");
                    let image = if let Some(image) = image_cache.get(&filepath).cloned() {
                        image
                    } else if let Some(i) = load_image(&filepath, false, Vector2::ONE).await {
                        image_cache.insert(filepath, i.clone());
                        i
                    } else {
                        break
                    };

                    frames.push(image.tex);
                    counter += 1;
                }
                if frames.len() == 0 { return Err(TatakuError::String("anim has no frames!".to_owned())) }

                let delays = vec![anim.frame_delay; frames.len()];

                let mut animation = Animation::new(Vector2::ZERO, 0.0, Vector2::ONE, frames, delays, Vector2::ONE);
                animation.scale = Vector2::ONE;
                animation.free_on_drop = true;
                
                let size = animation.size();
                match anim.origin {
                    Origin::TopLeft => animation.origin = Vector2::ZERO,
                    Origin::Centre => animation.origin = size / 2.0, // default
                    Origin::CentreLeft => animation.origin.x = 0.0,
                    Origin::TopRight => animation.origin = Vector2::new(size.x, 0.0),
                    Origin::BottomCentre => animation.origin.y = size.y,
                    Origin::TopCentre => animation.origin.y = 0.0,
                    Origin::Custom => animation.origin = Vector2::ZERO,
                    Origin::CentreRight => animation.origin.x = size.x,
                    Origin::BottomLeft => animation.origin = Vector2::new(0.0, size.y),
                    Origin::BottomRight => animation.origin = size,
                }

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

    fn apply_commands(&mut self, _scale: &ScalingHelper) {
        self.group.transforms.clear();

        let pos = match &self.def {
            StoryboardElementDef::Sprite(s) => s.pos,
            StoryboardElementDef::Animation(a) => a.pos,
        };
        self.group.pos.current = pos;

        // TODO: 
        // if these are wrong, they will be updated next frame anyways
        // self.group.pos.current = scale.scale_coords(pos);
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
            let trans_type = match i.event {
                StoryboardEvent::Move { start, end } => TransformType::Position { 
                    start,
                    end,
                },
                StoryboardEvent::MoveX { start_x, end_x } => TransformType::PositionX { 
                    start: start_x, 
                    end:   end_x,
                },
                StoryboardEvent::MoveY { start_y, end_y } => TransformType::PositionY { 
                    start: start_y, 
                    end:   end_y,
                },
                // StoryboardEvent::Move { start, end } => TransformType::Position { 
                //     start: scale.scale_coords(start), 
                //     end:   scale.scale_coords(end) 
                // },
                // StoryboardEvent::MoveX { start_x, end_x } => TransformType::PositionX { 
                //     start: scale.scale_coords(Vector2::with_x(start_x as f64)).x, 
                //     end:   scale.scale_coords(Vector2::with_x(end_x as f64)).x 
                // },
                // StoryboardEvent::MoveY { start_y, end_y } => TransformType::PositionY { 
                //     start: scale.scale_coords(Vector2::with_y(start_y as f64)).y, 
                //     end:   scale.scale_coords(Vector2::with_y(end_y as f64)).y 
                // },
                
                // uncomment these
                // StoryboardEvent::Scale { start_scale, end_scale } => TransformType::Scale { start: start_scale as f64, end: end_scale as f64 },
                // StoryboardEvent::VectorScale { start_scale, end_scale } => TransformType::VectorScale { start: start_scale, end: end_scale },

                StoryboardEvent::Fade { start, end } => TransformType::Transparency { start: start, end: end },
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

            let offset = i.start_time;
            let duration = i.end_time - i.start_time;
            earliest_start = earliest_start.min(i.start_time);
            latest_end = latest_end.max(i.end_time);
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

    fn update(&mut self, time: f32, _manager: &IngameManager) {
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

        b.layer.cmp(&a.layer) // should be correct
        // a.layer.cmp(&b.layer)
    }

}


enum ElementImage {
    Sprite(Image),
    Anim(Animation),
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