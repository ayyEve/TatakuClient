#![allow(unused)]
use crate::prelude::*;

macro_rules! chain {
    ($name: ident, $type:ty) => { pub fn $name(mut self, $name: $type) -> Self { self.$name = $name; self }}
}

/// helper for building emitters
/// useful if you have multiple emitters which only have one or two settings different between them
#[derive(Clone, Default)]
pub struct EmitterBuilder {
    spawn_delay: f32,
    position: Vector2,
    life: Range<f32>,
    angle: EmitterVal,
    speed: EmitterVal,
    scale: EmitterVal,
    opacity: EmitterVal,
    rotation: EmitterVal,
    color: Color,
    image: Arc<TextureReference>,
    should_emit: bool,
    blend_mode: BlendMode,
}
impl EmitterBuilder {
    pub fn new() -> Self { Self::default().should_emit(true) }

    pub fn build(self, time: f32) -> Emitter {
        let mut e = Emitter::new(time, self.spawn_delay, self.position, self.angle, self.speed, self.scale, self.life, self.opacity, self.rotation, self.color, self.image, self.blend_mode);
        e.should_emit = self.should_emit;
        e
    }

    chain!(spawn_delay, f32);
    chain!(position, Vector2);
    chain!(life, Range<f32>);
    chain!(angle, EmitterVal);
    chain!(speed, EmitterVal);
    chain!(scale, EmitterVal);
    chain!(opacity, EmitterVal);
    chain!(rotation, EmitterVal);
    chain!(color, Color);
    chain!(image, Arc<TextureReference>);
    chain!(should_emit, bool);
    chain!(blend_mode, BlendMode);
}
