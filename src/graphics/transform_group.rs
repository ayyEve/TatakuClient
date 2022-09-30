#![allow(dead_code)]
use crate::prelude::*;

#[derive(Clone)]
pub struct TransformGroup {
    pub items: Vec<DrawItem>,
    pub transforms: Vec<Transformation>
}
impl TransformGroup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            transforms: Vec::new()
        }
    }

    pub fn update(&mut self, game_time: f64) {
        let mut transforms = std::mem::take(&mut self.transforms);
        transforms.retain(|transform| {
            let start_time = transform.start_time();
            // transform hasnt started, ignore
            if game_time >= start_time {
                let trans_val = transform.get_value(game_time);
                for i in self.items.iter_mut() {
                    i.apply_transform(transform, trans_val.clone())
                }
            }

            game_time < start_time + transform.duration
        });

        self.transforms = transforms;
    }

    //TODO: maybe this could be improved?
    pub fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        list.reserve(self.items.len());
        for i in self.items.iter() {
            if !i.visible() { continue }
            list.push(i.to_renderable());
        }
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
            TransformType::BorderTransparency {start: 1.0, end: 0.0},
            TransformEasing::EaseOutSine,
            time
        ));

        // scale
        self.transforms.push(Transformation::new(
            0.0,
            duration * 1.1,
            TransformType::Scale {start: 1.0, end: end_scale},
            TransformEasing::Linear,
            time
        ));

        // border size
        if do_border_size {
            self.transforms.push(Transformation::new(
                0.0,
                duration * 1.1,
                TransformType::BorderSize {start: 2.0, end: 0.0},
                TransformEasing::EaseInSine,
                time
            ));
        }
    }

    #[allow(unused)] // will be used eventually probably
    pub fn ripple_scale_range(&mut self, offset:f64, duration:f64, time: f64, scale: Range<f64>, border_size: Option<Range<f64>>, do_transparency: Option<f32>) {
        
        // border transparency
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
            0.0,
            duration * 1.1,
            TransformType::Scale { start: scale.start, end: scale.end },
            TransformEasing::Linear,
            time
        ));

        // border size
        if let Some(b) = border_size {
            self.transforms.push(Transformation::new(
                0.0,
                duration * 1.1,
                TransformType::BorderSize { start: b.start, end: b.end },
                TransformEasing::EaseInSine,
                time
            ));
        }
    }
}

#[derive(Clone)]
pub enum DrawItem {
    Line(Line),
    Text(Text),
    Image(Image),
    Circle(Circle),
    Rectangle(Rectangle),
    HalfCircle(HalfCircle),
    SkinnedNumber(SkinnedNumber)
}
impl DrawItem {
    pub fn get_pos(&self) -> Vector2 {
        match self {
            // DrawItem::Line(a) => a.current_pos,
            DrawItem::Text(a) => a.current_pos,
            DrawItem::Image(a) => a.current_pos,
            DrawItem::Circle(a) => a.current_pos,
            DrawItem::Rectangle(a) => a.current_pos,
            DrawItem::SkinnedNumber(a) => a.current_pos,
            // DrawItem::HalfCircle(a) => a.current_pos,
            _ => Vector2::zero()
        }
    }

    pub fn apply_transform(&mut self, transform: &Transformation, trans_val: TransformValueResult) {
        match self {
            // DrawItem::Line(a) => a.apply_transform(transform, trans_val),
            DrawItem::Text(a) => a.apply_transform(transform, trans_val),
            DrawItem::Image(a) => a.apply_transform(transform, trans_val),
            DrawItem::Circle(a) => a.apply_transform(transform, trans_val),
            DrawItem::Rectangle(a) => a.apply_transform(transform, trans_val),
            DrawItem::SkinnedNumber(a) => a.apply_transform(transform, trans_val),
            // DrawItem::HalfCircle(a) => a.apply_transform(transform, trans_val),
            _ => {}
        };
    }

    pub fn to_renderable(&self) -> Box<dyn Renderable> {
        let new_item:Box<dyn Renderable> = match self {
            DrawItem::Line(a) => Box::new(a.clone()),
            DrawItem::Text(a) => Box::new(a.clone()),
            DrawItem::Image(a) => Box::new(a.clone()),
            DrawItem::Circle(a) => Box::new(a.clone()),
            DrawItem::Rectangle(a) => Box::new(a.clone()),
            DrawItem::HalfCircle(a) => Box::new(a.clone()),
            DrawItem::SkinnedNumber(a) => Box::new(a.clone()),
        };

        new_item
    }

    pub fn visible(&self) -> bool {
        match self {
            // DrawItem::Line(a) => a.visible(),
            DrawItem::Text(a) => a.visible(),
            DrawItem::Image(a) => a.visible(),
            DrawItem::Circle(a) => a.visible(),
            DrawItem::Rectangle(a) => a.visible(),
            DrawItem::SkinnedNumber(a) => a.visible(),
            // DrawItem::HalfCircle(a) => a.visible(),
            _ => { true }
        }
    }
}