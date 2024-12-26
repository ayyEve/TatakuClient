use crate::prelude::*;

#[derive(Clone)]
pub struct TransformManager {
    pub pos: InitialCurrent<Vector2>,
    pub scale: InitialCurrent<Vector2>,
    pub rotation: InitialCurrent<f32>,
    pub alpha: InitialCurrent<f32>,
    pub border_alpha: InitialCurrent<f32>,

    pub color: Option<InitialCurrent<Color>>,
    pub border_color: Option<InitialCurrent<Color>>,

    pub origin: Vector2,

    pub image_flip_horizonal: bool,
    pub image_flip_vertical: bool,


    pub transforms: Vec<Transformation>,
}

impl TransformManager {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos: InitialCurrent::new(pos),
            scale: InitialCurrent::new(Vector2::ONE),
            rotation: InitialCurrent::new(0.0),
            alpha: InitialCurrent::new(1.0),
            border_alpha: InitialCurrent::new(1.0),

            color: None,
            border_color: None,

            origin: Vector2::ZERO,
            
            image_flip_horizonal: false,
            image_flip_vertical: false,
            
            transforms: Vec::new(),
        }
    }

    pub fn pos(mut self, pos: Vector2) -> Self {
        self.pos.both(pos);
        self
    }
    pub fn scale(mut self, scale: Vector2) -> Self {
        self.scale.both(scale);
        self
    }
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation.both(rotation);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha.both(alpha);
        self
    }
    pub fn border_alpha(mut self, alpha: f32) -> Self {
        self.border_alpha.both(alpha);
        self
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

    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                self.pos.current = self.pos.initial + val;
            }
            TransformType::PositionX { .. } => {
                let val:f64 = val.into();
                self.pos.current.x = self.pos.initial.x + val as f32;
            }
            TransformType::PositionY { .. } => {
                let val:f64 = val.into();
                self.pos.current.y = self.pos.initial.y + val as f32;
            }
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.scale.current = Vector2::ONE * val as f32;

                if self.image_flip_horizonal {
                    self.scale.current.x *= -1.0;
                }
                if self.image_flip_vertical {
                    self.scale.current.y *= -1.0;
                }
            }
            TransformType::VectorScale { .. } => {
                let val:Vector2 = val.into();
                self.scale.current = val;

                if self.image_flip_horizonal {
                    self.scale.current.x *= -1.0;
                }
                if self.image_flip_vertical {
                    self.scale.current.y *= -1.0;
                }
            }
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.rotation.current = self.rotation.initial + val as f32;
            }

            TransformType::Transparency { .. } => {
                let val:f64 = val.into();
                self.alpha.current = val as f32;
            }
            TransformType::BorderTransparency { .. } => {
                let val:f64 = val.into();
                self.border_alpha.current = val as f32;
            }

            TransformType::Color { .. } => {
                let color:Color = val.into();
                match &mut self.color {
                    Some(a) => a.current = color,
                    None => self.color = Some(InitialCurrent::new(color)),
                }
            }

            _ => {}
        }
    }


    pub fn visible(&self) -> bool {
        self.scale.length_squared() != 0.0 && (*self.alpha > 0.0 || *self.border_alpha > 0.0)
    }

    pub fn matrix(&self) -> Matrix {
        Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(*self.rotation) // rotate
            .scale(*self.scale) // scale
            .trans(*self.pos) // move to pos
    }
}


// premade transforms
impl TransformManager {
    pub fn ripple(&mut self, offset:f32, duration:f32, time: f32, end_scale: f32, do_border_size: bool, do_transparency: Option<f32>) {
        
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

    pub fn ripple_scale_range(&mut self, offset:f32, duration:f32, time: f32, scale: Range<f32>, border_size: Option<Range<f32>>, do_transparency: Option<f32>) {
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

    pub fn shake(&mut self, offset:f32, time: f32, shake_amount: Vector2, time_between_shakes: f32, shake_count: usize) {
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