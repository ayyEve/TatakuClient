use crate::prelude::*;

#[derive(Clone)]
#[derive(ChainableInitializer)]
pub struct TransformManager {
    #[chain] pub pos: Vector2,
    #[chain] pub scale: Vector2,
    #[chain] pub rotation: f32,
    #[chain] pub origin: Vector2,

    #[chain] pub alpha: f32,
    #[chain] pub border_alpha: f32,
    #[chain] pub color: Option<Color>,
    #[chain] pub border_color: Option<Color>,

    #[chain] pub image_flip_horizonal: bool,
    #[chain] pub image_flip_vertical: bool,

    pub transforms: Vec<Transformation>,
}

impl TransformManager {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            scale: Vector2::ONE,
            rotation: 0.0,
            alpha: 1.0,
            border_alpha: 1.0,

            color: None,
            border_color: None,

            origin: Vector2::ZERO,

            image_flip_horizonal: false,
            image_flip_vertical: false,

            transforms: Vec::new(),
        }
    }


    pub fn push_transform(&mut self, transform: Transformation) {
        self.transforms.push(transform);
    }

    pub fn update(&mut self, game_time: f32) {
        let mut transforms = self.transforms.take();
        transforms.retain(|transform| {
            let start_time = transform.start_time();
            let end_time = start_time + transform.duration;

            if game_time >= end_time {
                let trans_val = transform.get_value(end_time);
                self.apply_transform(transform, trans_val);
            } else if game_time >= start_time {
                let trans_val = transform.get_value(game_time);
                self.apply_transform(transform, trans_val);
            }

            game_time < end_time
        });

        self.transforms = transforms;
    }

    fn apply_transform(
        &mut self, 
        transform: &Transformation, 
        val: TransformValueResult
    ) {
        match transform.trans_type {
            TransformType::None => {},
            TransformType::BorderSize { .. } => {}

            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                self.pos = val;
            }
            TransformType::PositionX { .. } => {
                let val:f64 = val.into();
                self.pos.x = val as f32;
            }
            TransformType::PositionY { .. } => {
                let val:f64 = val.into();
                self.pos.y = val as f32;
            }
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.scale = Vector2::ONE * val as f32;

                // if self.image_flip_horizonal {
                //     self.scale.x *= -1.0;
                // }
                // if self.image_flip_vertical {
                //     self.scale.y *= -1.0;
                // }
            }
            TransformType::ScaleX { .. } => {
                let val:f64 = val.into();
                self.scale.x = val as f32;

                // if self.image_flip_horizonal {
                //     self.scale.x *= -1.0;
                // }
            }
            TransformType::ScaleY { .. } => {
                let val:f64 = val.into();
                self.scale.y = val as f32;

                if self.image_flip_vertical {
                    self.scale.y *= -1.0;
                }
            }
            TransformType::VectorScale { .. } => {
                let val:Vector2 = val.into();
                self.scale = val;

                // if self.image_flip_horizonal {
                //     self.scale.x *= -1.0;
                // }
                // if self.image_flip_vertical {
                //     self.scale.y *= -1.0;
                // }
            }
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.rotation = val as f32;
            }

            TransformType::Transparency { .. } => {
                let val:f64 = val.into();
                self.alpha = val as f32;
            }
            TransformType::BorderTransparency { .. } => {
                let val:f64 = val.into();
                self.border_alpha = val as f32;
            }

            TransformType::Color { .. } => {
                let color:Color = val.into();
                match &mut self.color {
                    Some(a) => *a = color,
                    None => self.color = Some(color),
                }
            }

            // _ => {}
        }
    }


    pub fn visible(&self) -> bool {
        self.scale.length_squared() != 0.0 && (self.alpha > 0.0 || self.border_alpha > 0.0)
    }

    pub fn matrix(&self) -> Matrix {
        Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale) // scale
            .trans(self.pos) // move to pos
    }
}


// premade transforms
impl TransformManager {
    pub fn ripple(
        &mut self, 
        offset: f32, 
        duration: f32, 
        time: f32, 
        end_scale: f32, 
        do_border_size: bool, 
        do_transparency: Option<f32>
    ) {
        // transparency
        if let Some(start_a) = do_transparency {
            self.transforms.push(Transformation::new(
                offset,
                duration,
                TransformType::Transparency {start: start_a, end: 0.0},
                Easing::EaseOutSine,
                time
            ));
        }

        // border transparency
        self.transforms.push(Transformation::new(
            offset,
            duration,
            TransformType::BorderTransparency { start: 1.0, end: 0.0 },
            Easing::EaseOutSine,
            time
        ));

        // scale
        self.transforms.push(Transformation::new(
            offset,
            duration * 1.1,
            TransformType::Scale {start: 1.0, end: end_scale},
            Easing::Linear,
            time
        ));

        // border size
        if do_border_size {
            self.transforms.push(Transformation::new(
                offset,
                duration * 1.1,
                TransformType::BorderSize {start: 2.0, end: 0.0},
                Easing::EaseInSine,
                time
            ));
        }
    }

    pub fn ripple_scale_range(
        &mut self, 
        offset: f32, 
        duration: f32, 
        time: f32, 
        scale: Range<f32>, 
        border_size: Option<Range<f32>>, 
        do_transparency: Option<f32>
    ) {
        // transparency
        if let Some(start_a) = do_transparency {
            self.transforms.push(Transformation::new(
                offset,
                duration,
                TransformType::Transparency { start: start_a, end: 0.0 },
                Easing::EaseOutSine,
                time
            ));
        }

        // border transparency
        self.transforms.push(Transformation::new(
            offset,
            duration,
            TransformType::BorderTransparency { start: 1.0, end: 0.0 },
            Easing::EaseOutSine,
            time
        ));

        // scale
        self.transforms.push(Transformation::new(
            offset,
            duration * 1.1,
            TransformType::Scale { start: scale.start, end: scale.end },
            Easing::EaseOutQuadratic,
            time
        ));

        // border size
        if let Some(b) = border_size {
            self.transforms.push(Transformation::new(
                offset,
                duration * 1.1,
                TransformType::BorderSize { start: b.start, end: b.end },
                Easing::EaseInSine,
                time
            ));
        }
    }

    pub fn shake(
        &mut self, 
        offset:f32, 
        time: f32, 
        shake_amount: Vector2, 
        time_between_shakes: f32, 
        shake_count: usize
    ) {
        self.transforms.reserve(shake_count);

        self.transforms.push(Transformation::new(
            offset,
            time_between_shakes,
            TransformType::Position { start: Vector2::ZERO, end: shake_amount },
            Easing::Linear,
            time
        ));

        if shake_count > 2 {
            for i in 0..shake_count-2 {
                let pos = if i % 2 == 0 {
                    TransformType::Position { start: shake_amount, end: -shake_amount }
                } else {
                    TransformType::Position { start: -shake_amount, end: shake_amount }
                };

                self.transforms.push(Transformation::new(
                    offset + (time_between_shakes * (i+1) as f32),
                    time_between_shakes,
                    pos,
                    Easing::Linear,
                    time
                ));
            }
        }

        let end_pos = if shake_count % 2 == 0 {
            TransformType::Position { start: -shake_amount, end: Vector2::ZERO }
        } else {
            TransformType::Position { start: shake_amount, end: Vector2::ZERO }
        };

        self.transforms.push(Transformation::new(
            offset + (time_between_shakes * (shake_count+2) as f32),
            time_between_shakes,
            end_pos,
            Easing::Linear,
            time
        ));
    }
}

impl Default for TransformManager {
    fn default() -> Self {
        Self::new(Vector2::ZERO)
    }
}
