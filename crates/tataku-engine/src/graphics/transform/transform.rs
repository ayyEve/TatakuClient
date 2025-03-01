use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
    pub origin: Vector2,
}
impl Transform {
    pub fn new(
        pos: Vector2, 
        scale: Vector2, 
        rotation: f32, 
        origin: Vector2
    ) -> Self {
        Self {
            pos,
            scale,
            rotation,
            origin
        }
    }

    pub fn from_manager(manager: &TransformManager) -> Self {
        Self::new(
            manager.pos,
            manager.scale,
            manager.rotation,
            manager.origin
        )
    }

    pub fn matrix(&self) -> Matrix {
        Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale) // scale
            .trans(self.pos) // move to pos
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vector2::ZERO,
            scale: Vector2::ONE,
            rotation: 0.0,
            origin: Vector2::ZERO
        }
    }
}

pub struct TransformedDrawable {
    pub transform: Transform,
    pub drawable: Box<dyn TatakuRenderable>
}
impl TransformedDrawable {
    pub fn new(
        transform: Transform,
        drawable: Box<dyn TatakuRenderable>
    ) -> Self {
        Self {
            transform,
            drawable
        }
    }
}
impl TatakuRenderable for TransformedDrawable {
    fn get_bounds(&self) -> Bounds {
        self.drawable.get_bounds()
    }

    fn get_blend_mode(&self) -> BlendMode {
        self.drawable.get_blend_mode()
    }

    fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.drawable.set_blend_mode(blend_mode);
    }

    fn get_scissor(&self) -> Scissor {
        self.drawable.get_scissor()
    }
    fn set_scissor(&mut self, c: Scissor) {
        self.drawable.set_scissor(c);
    }

    fn draw(
        &self,
        options: &DrawOptions,
        mut transform: Matrix,
        g: &mut dyn GraphicsEngine,
    ) {
        transform = transform * self.transform.matrix();
        self.drawable.draw(options, transform, g)
    }
}


pub struct ScissoredDrawable {
    pub scissor: [f32; 4],
    pub drawable: Box<dyn TatakuRenderable>
}
impl ScissoredDrawable {
    pub fn new(
        scissor: [f32; 4],
        drawable: Box<dyn TatakuRenderable>
    ) -> Self {
        Self {
            scissor,
            drawable
        }
    }
}
impl TatakuRenderable for ScissoredDrawable {
    fn get_bounds(&self) -> Bounds {
        self.drawable.get_bounds()
    }

    fn get_blend_mode(&self) -> BlendMode {
        self.drawable.get_blend_mode()
    }

    fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.drawable.set_blend_mode(blend_mode);
    }

    fn get_scissor(&self) -> Scissor {
        Some(self.scissor)
    }
    fn set_scissor(&mut self, s: Scissor) {
        self.drawable.set_scissor(s);
    }

    fn draw(
        &self,
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn GraphicsEngine,
    ) {
        g.push_scissor(self.scissor);
        self.drawable.draw(options, transform, g);
        g.pop_scissor();
    }
}
