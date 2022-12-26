#![allow(dead_code)]
use crate::prelude::*;

#[derive(Clone)]
pub struct TransformGroup {
    pub pos: InitialCurrent<Vector2>,
    pub scale: InitialCurrent<Vector2>,
    pub rotation: InitialCurrent<f64>,
    pub alpha: InitialCurrent<f32>,
    pub border_alpha: InitialCurrent<f32>,

    pub depth: f64,
    pub origin: Vector2,

    pub items: Vec<Arc<dyn TatakuRenderable>>,
    pub transforms: Vec<Transformation>,

    pub draw_state: Option<DrawState>,
}
impl TransformGroup {
    pub fn new(pos: Vector2, depth: f64) -> Self {
        Self {
            items: Vec::new(),
            transforms: Vec::new(),
            origin: Vector2::ZERO,

            depth, 
            draw_state: None,

            pos: InitialCurrent::new(pos),
            scale: InitialCurrent::new(Vector2::ONE),
            rotation: InitialCurrent::new(0.0),
            alpha: InitialCurrent::new(1.0),
            border_alpha: InitialCurrent::new(1.0)
        }
    }

    pub fn scale(mut self, scale: Vector2) -> Self {
        self.scale.both(scale);
        self
    }
    pub fn rotation(mut self, rotation: f64) -> Self {
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
    


    pub fn update(&mut self, game_time: f64) {
        let mut transforms = std::mem::take(&mut self.transforms);
        transforms.retain(|transform| {
            let start_time = transform.start_time();
            // transform hasnt started, ignore
            if game_time >= start_time {
                let trans_val = transform.get_value(game_time);
                self.apply_transform(transform, trans_val);
            }

            game_time < start_time + transform.duration
        });

        self.transforms = transforms;
    }

    // //TODO: maybe this could be improved?
    // pub fn draw(&mut self, list: &mut RenderableCollection) {
    //     // list.reserve(self.items.len());
    //     for i in self.items.iter() {
    //         if !i.visible() { continue }

    //         match i {
    //             DrawItem::Line(a) => list.push(a.clone()),
    //             DrawItem::Text(a) => list.push(a.clone()),
    //             DrawItem::Image(a) => list.push(a.clone()),
    //             DrawItem::Circle(a) => list.push(a.clone()),
    //             DrawItem::Rectangle(a) => list.push(a.clone()),
    //             DrawItem::HalfCircle(a) => list.push(a.clone()),
    //             DrawItem::SkinnedNumber(a) => list.push(a.clone()),
    //         }
    //     }
    // }

    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                self.pos.current = self.pos.initial + val;
            }
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.scale.current = Vector2::ONE * val;
            }
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.rotation.current = self.rotation.initial + val;
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
    pub fn ripple(&mut self, offset:f64, duration:f64, time: f64, end_scale: f64, do_border_size: bool, do_transparency: Option<f32>) {
        
        // transparency
        if let Some(start_a) = do_transparency {
            self.transforms.push(Transformation::new(
                offset,
                duration,
                TransformType::Transparency {start: start_a as f64, end: 0.0},
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
    pub fn ripple_scale_range(&mut self, offset:f64, duration:f64, time: f64, scale: Range<f64>, border_size: Option<Range<f64>>, do_transparency: Option<f32>) {
        
        // transparency
        if let Some(start_a) = do_transparency {
            self.transforms.push(Transformation::new(
                offset,
                duration,
                TransformType::Transparency { start: start_a as f64, end: 0.0 },
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


impl Renderable for TransformGroup {
    fn get_depth(&self) -> f64 { self.depth }

    fn draw(&self, g: &mut GlGraphics, mut c:Context) {
        if let Some(d) = self.draw_state {
            c.draw_state = d;
        }

        let pre_rotation = self.pos.current / self.scale.current + self.origin;

        c.transform = c.transform
            // scale to size
            .scale_pos(*self.scale)

            // move to pos
            .trans_pos(pre_rotation)

            // rotate to rotate
            .rot_rad(*self.rotation)
            
            // apply origin
            .trans_pos(-self.origin)
        ;
        
        for i in self.items.iter() {
            i.draw_with_transparency(c, *self.alpha, *self.border_alpha, g)
        }
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