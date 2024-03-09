use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Image {
    // pub size: Vector2,
    pub tex: TextureReference,
    /// underlying scale of this image, mainly used for 2x res sprites
    pub base_scale: Vector2,

    /// origin of rotation in px, relative to image position
    pub origin: Vector2,

    scissor: Scissor,
    blend_mode: BlendMode,

    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
}
impl Image {
    pub fn new(pos:Vector2, tex:TextureReference, base_scale: Vector2) -> Image {
        // let scale = Vector2::new(tex.get_width() as f64 / size.x, tex.get_height() as f64 / size.y);
        let tex_size = Vector2::new(tex.width as f32, tex.height as f32);

        let rotation = 0.0;
        let color = Color::WHITE;

        let origin = tex_size / 2.0;

        Image {
            pos,
            scale: Vector2::ONE,
            rotation,
            color,

            // size: tex_size,
            origin,
            tex,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
            base_scale
        }
    }

    pub fn size(&self) -> Vector2 {
        self.tex_size() * self.scale
    }
    pub fn set_size(&mut self, size: Vector2) {
        let tex_size = self.tex_size();
        self.scale = size / tex_size;
    }

    fn raw_tex_size(&self) -> Vector2 {
        Vector2::new(self.tex.width as f32, self.tex.height as f32)
    }
    pub fn tex_size(&self) -> Vector2 { 
        self.raw_tex_size() * self.base_scale
    }

    pub fn fit_to_bg_size(&mut self, size: Vector2, center: bool) {
        // resize to maintain aspect ratio
        let image_size = self.tex_size();
        let ratio = image_size.y / image_size.x;

        if image_size.x > image_size.y {
            // use width as base
            self.set_size(Vector2::new(
                size.x,
                size.x * ratio
            ));
        } else {
            // use height as base
            self.set_size(Vector2::new(
                size.y * ratio,
                size.y
            ));
        }

        if center {
            self.origin = self.raw_tex_size() / 2.0;
            self.pos = size / 2.0;
        } else {
            self.origin = Vector2::ZERO;
            self.pos = (size - self.size()) / 2.0;
        }
    }
}

impl TatakuRenderable for Image {
    fn get_name(&self) -> String { "Texture".to_owned() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, self.size()) }
    
    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s:Scissor) { self.scissor = s }
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, mut transform: Matrix, g: &mut GraphicsState) {
        let scale = self.scale * self.base_scale;
        let h_flip = false;
        let v_flip = false;

        // if scale.x < 0.0 {
        //     scale.x = scale.x.abs();
        //     h_flip = true;
        // }
        // if scale.y < 0.0 {
        //     scale.y = scale.y.abs();
        //     v_flip = true;
        // }

        transform = transform * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(scale) // scale
            .trans(self.pos) // move to pos
        ;

        g.draw_tex(&self.tex, self.color.alpha(alpha), h_flip, v_flip, transform, self.blend_mode);
    }
}
