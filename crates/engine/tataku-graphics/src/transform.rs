use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default2)]
pub struct Transform {
    pub origin: Vector2,
    #[default(Vector2::ONE)]
    pub scale: Vector2,
    pub rotation: f32,
    pub pos: Vector2,
}
impl Transform {
    pub fn new(
        pos: Vector2,
        scale: Vector2,
        rotation: f32,
        origin: Vector2
    ) -> Self {
        Self {
            origin,
            scale,
            rotation,
            pos,
        }
    }

    pub fn rotate(self, angle: f32) -> Self {
        let x = angle.cos() * self.pos.x - angle.sin() * self.pos.y;
        let y = angle.sin() * self.pos.x + angle.cos() * self.pos.y;

        Self {
            rotation: self.rotation + angle,
            pos: Vector2::new(x, y),
            ..self
        }
    }

    pub fn scale(self, scale: Vector2) -> Self {
        Self {
            scale: self.scale * scale,
            pos: self.pos * scale,
            ..self
        }
    }

    pub fn translate(self, trans: Vector2) -> Self {
        Self {
            pos: self.pos + trans,
            ..self
        }
    }

    pub fn matrix(&self) -> Matrix {
        Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale) // scale
            .trans(self.pos) // move to pos
    }

    pub fn aabb_bounds(&self, bounds: Bounds) -> Bounds {
        let matrix = self.matrix();

        let pos = matrix * bounds.pos;
        let br = matrix * (bounds.pos + bounds.size);

        // Calculate AABB
        let min_x = pos.x.min(br.x);
        let min_y = pos.y.min(br.y);
        let max_x = pos.x.max(br.x);
        let max_y = pos.y.max(br.y);

        let pos = Vector2::new(min_x, min_y);
        let br = Vector2::new(max_x, max_y);

        Bounds::new(pos, br - pos)
    }
}

pub struct Transformed {
    pub transform: Matrix,
    pub drawable: Box<dyn TatakuRenderable>,
}
impl Transformed {
    pub fn new(
        transform: Transform,
        drawable: Box<dyn TatakuRenderable>,
    ) -> Self {
        Self {
            transform: transform.matrix(),
            drawable,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for Transformed {
    fn get_blend_mode(&self) -> GraphicsPipeline {
        self.drawable.get_blend_mode()
    }

    fn set_blend_mode(&mut self, blend_mode: GraphicsPipeline) {
        self.drawable.set_blend_mode(blend_mode);
    }

    fn draw(
        &self,
        options: &DrawOptions,
        mut transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        transform = transform * self.transform;
        self.drawable.draw(options, transform, g);
    }
}


pub struct Scissored {
    pub scissor: [f32; 4],
    pub drawable: Box<dyn TatakuRenderable>
}
impl Scissored {
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

#[cfg(feature="graphics")]
impl TatakuRenderable for Scissored {
    fn get_blend_mode(&self) -> GraphicsPipeline {
        self.drawable.get_blend_mode()
    }

    fn set_blend_mode(&mut self, blend_mode: GraphicsPipeline) {
        self.drawable.set_blend_mode(blend_mode);
    }

    fn draw(
        &self,
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        g.push_scissor(self.scissor);
        self.drawable.draw(options, transform, g);
        g.pop_scissor();
    }
}

pub struct MergeDrawOptions {
    pub draw_options: DrawOptions,
    pub drawable: Box<dyn TatakuRenderable>
}
impl MergeDrawOptions {
    pub fn new(
        draw_options: DrawOptions,
        drawable: Box<dyn TatakuRenderable>
    ) -> Self {
        Self {
            draw_options,
            drawable
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for MergeDrawOptions {
    fn get_blend_mode(&self) -> GraphicsPipeline {
        self.drawable.get_blend_mode()
    }

    fn set_blend_mode(&mut self, blend_mode: GraphicsPipeline) {
        self.drawable.set_blend_mode(blend_mode);
    }

    fn draw(
        &self,
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        let options = options.merge(self.draw_options);
        self.drawable.draw(&options, transform, g);
    }
}
