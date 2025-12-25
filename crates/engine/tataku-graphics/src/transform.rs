use crate::*;

#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub origin: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
    pub pos: Vector2,
}
impl Transform {
    pub const fn new(
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

    pub const fn identity() -> Self {
        Self {
            origin: Vector2::ZERO,
            scale: Vector2::ONE,
            rotation: 0.0,
            pos: Vector2::ZERO,
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

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

pub struct Transformed<T = Box<dyn TatakuRenderable>> {
    pub transform: Matrix,
    pub drawable: T,
}
impl<T> Transformed<T> {
    pub fn new(
        transform: Transform,
        drawable: T,
    ) -> Self {
        Self {
            transform: transform.matrix(),
            drawable,
        }
    }
}

#[cfg(feature="graphics")]
impl<T: TatakuRenderable> TatakuRenderable for Transformed<T> {
    fn get_name(&self) -> String { self.drawable.get_name() }

    fn get_pipeline(&self) -> GraphicsPipeline {
        self.drawable.get_pipeline()
    }

    fn set_pipeline(&mut self, blend_mode: GraphicsPipeline) {
        self.drawable.set_pipeline(blend_mode);
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


pub struct Scissored<T = Box<dyn TatakuRenderable>> {
    pub scissor: Bounds,
    pub drawable: T
}
impl<T> Scissored<T> {
    pub fn new(
        scissor: Bounds,
        drawable: T
    ) -> Self {
        Self {
            scissor,
            drawable
        }
    }
}

#[cfg(feature="graphics")]
impl<T: TatakuRenderable> TatakuRenderable for Scissored<T> {
    fn get_name(&self) -> String { self.drawable.get_name() }

    fn get_pipeline(&self) -> GraphicsPipeline {
        self.drawable.get_pipeline()
    }

    fn set_pipeline(&mut self, blend_mode: GraphicsPipeline) {
        self.drawable.set_pipeline(blend_mode);
    }

    fn draw(
        &self,
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        let scissor = transform * self.scissor;
        g.push_scissor(scissor.into_scissor());
        self.drawable.draw(options, transform, g);
        g.pop_scissor();
    }
}

pub struct MergeDrawOptions<T = Box<dyn TatakuRenderable>> {
    pub draw_options: DrawOptions,
    pub drawable: T
}
impl<T> MergeDrawOptions<T> {
    pub fn new(
        draw_options: DrawOptions,
        drawable: T,
    ) -> Self {
        Self {
            draw_options,
            drawable
        }
    }
}

#[cfg(feature="graphics")]
impl<T: TatakuRenderable> TatakuRenderable for MergeDrawOptions<T> {
    fn get_name(&self) -> String { self.drawable.get_name() }

    fn get_pipeline(&self) -> GraphicsPipeline {
        self.drawable.get_pipeline()
    }

    fn set_pipeline(&mut self, blend_mode: GraphicsPipeline) {
        self.drawable.set_pipeline(blend_mode);
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
