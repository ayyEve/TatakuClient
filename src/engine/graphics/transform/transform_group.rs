#![allow(dead_code)]
use crate::prelude::*;

#[derive(Clone)]
pub struct TransformGroup {
    pub pos: InitialCurrent<Vector2>,
    pub scale: InitialCurrent<Vector2>,
    pub rotation: InitialCurrent<f32>,
    pub alpha: InitialCurrent<f32>,
    pub border_alpha: InitialCurrent<f32>,

    pub depth: f32,
    pub origin: Vector2,

    pub items: Vec<Arc<dyn TatakuRenderable>>,
    pub transforms: Vec<Transformation>,

    // pub draw_state: Option<DrawState>,
    
    pub image_flip_horizonal: bool,
    pub image_flip_vertical: bool,
}
impl TransformGroup {
    pub fn new(pos: Vector2, depth: f32) -> Self {
        Self {
            items: Vec::new(),
            transforms: Vec::new(),
            origin: Vector2::ZERO,

            depth, 
            // draw_state: None,

            pos: InitialCurrent::new(pos),
            scale: InitialCurrent::new(Vector2::ONE),
            rotation: InitialCurrent::new(0.0),
            alpha: InitialCurrent::new(1.0),
            border_alpha: InitialCurrent::new(1.0),

            image_flip_horizonal: false,
            image_flip_vertical: false,
        }
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
    


    pub fn update(&mut self, game_time: f32) {
        let mut transforms = std::mem::take(&mut self.transforms);
        transforms.retain(|transform| {
            let start_time = transform.start_time();
            let end_time = start_time + transform.duration;

            // if the transform hasnt started, ignore
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

            _ => {}
        }
    }

    pub fn visible(&self) -> bool {
        self.scale.length_squared() != 0.0 && (*self.alpha > 0.0 || *self.border_alpha > 0.0)
    }

    pub fn push(&mut self, r: impl TatakuRenderable + 'static) {
        self.items.push(Arc::new(r));
    }
}

// premade transforms
impl TransformGroup {
    pub fn ripple(&mut self, offset:f32, duration:f32, time: f32, end_scale: f32, do_border_size: bool, do_transparency: Option<f32>) {
        
        // transparency
        if let Some(start_a) = do_transparency {
            self.transforms.push(Transformation::new(
                offset,
                duration,
                TransformType::Transparency {start: start_a, end: 0.0},
                TransformEasing::EaseOutSine,
                time
            ));
        }
        
        // border transparency
        self.transforms.push(Transformation::new(
            offset,
            duration,
            TransformType::BorderTransparency { start: 1.0, end: 0.0 },
            TransformEasing::EaseOutSine,
            time
        ));

        // scale
        self.transforms.push(Transformation::new(
            offset,
            duration * 1.1,
            TransformType::Scale {start: 1.0, end: end_scale},
            TransformEasing::Linear,
            time
        ));

        // border size
        if do_border_size {
            self.transforms.push(Transformation::new(
                offset,
                duration * 1.1,
                TransformType::BorderSize {start: 2.0, end: 0.0},
                TransformEasing::EaseInSine,
                time
            ));
        }
    }

    #[allow(unused)] // will be used eventually probably
    pub fn ripple_scale_range(&mut self, offset:f32, duration:f32, time: f32, scale: Range<f32>, border_size: Option<Range<f32>>, do_transparency: Option<f32>) {
        
        // transparency
        if let Some(start_a) = do_transparency {
            self.transforms.push(Transformation::new(
                offset,
                duration,
                TransformType::Transparency { start: start_a, end: 0.0 },
                TransformEasing::EaseOutSine,
                time
            ));
        }

        // border transparency
        self.transforms.push(Transformation::new(
            offset,
            duration,
            TransformType::BorderTransparency { start: 1.0, end: 0.0 },
            TransformEasing::EaseOutSine,
            time
        ));

        // scale
        self.transforms.push(Transformation::new(
            offset,
            duration * 1.1,
            TransformType::Scale { start: scale.start, end: scale.end },
            TransformEasing::Linear,
            time
        ));

        // border size
        if let Some(b) = border_size {
            self.transforms.push(Transformation::new(
                offset,
                duration * 1.1,
                TransformType::BorderSize { start: b.start, end: b.end },
                TransformEasing::EaseInSine,
                time
            ));
        }
    }
}


impl TatakuRenderable for TransformGroup {
    fn get_depth(&self) -> f32 { self.depth }

    fn draw(&self, mut transform: Matrix, g: &mut GraphicsState) {
        transform = transform
            * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(*self.rotation) // rotate
            .scale(*self.scale) // scale
            .trans(*self.pos) // move to pos
        ;

        
        for i in self.items.iter() {
            i.draw_with_transparency(*self.alpha, *self.border_alpha, transform, g)
        }
    }

    fn draw_with_transparency(&self, _alpha: f32, _border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        self.draw(transform, g)
    }
}


pub struct InitialCurrent<T> {
    pub initial: T,
    pub current: T,
}
impl<T:Clone> InitialCurrent<T> {
    pub fn new(val: T) -> Self {
        Self {
            initial: val.clone(),
            current: val,
        }
    }
    pub fn both(&mut self, val: T) {
        self.initial = val.clone();
        self.current = val;
    }
}
impl<T> Deref for InitialCurrent<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.current
    }
}
impl<T:Clone> Clone for InitialCurrent<T> {
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            initial: self.initial.clone(),
        }
    }
}
impl<T:Copy> Copy for InitialCurrent<T> {}
