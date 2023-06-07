use crate::prelude::*;

#[derive(Clone)]
pub struct Image {
    // pub size: Vector2,
    pub depth: f32,
    pub tex: TextureReference,
    /// underlying scale of this image, mainly used for 2x res sprites
    pub base_scale: Vector2,

    // origin of rotation in px, relative to image position
    pub origin: Vector2,

    // draw_state: Option<DrawState>,

    pub color: Color,
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
}
impl Image {
    pub fn new(pos:Vector2, depth:f32, tex:TextureReference, base_scale: Vector2) -> Image {
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
            depth,
            origin,
            tex,
            // draw_state: None,
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
    fn get_depth(&self) -> f32 { self.depth }

    // fn get_draw_state(&self) -> Option<DrawState> { self.draw_state }
    // fn set_draw_state(&mut self, c:Option<DrawState>) { self.draw_state = c }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        let mut scale = self.scale;
        let mut h_flip = false;
        let mut v_flip = false;

        if scale.x < 0.0 {
            scale.x = scale.x.abs();
            h_flip = true;
        }
        if scale.y < 0.0 {
            scale.y = scale.y.abs();
            v_flip = true;
        }

        let transform = transform
            // apply origin
            .trans(-self.origin)

            // rotate to rotate
            .rot(self.rotation)

            // scale to size
            .scale(scale)

            // move to pos
            .trans(self.pos)
        ;

        g.draw_tex(&self.tex, self.depth, self.color.alpha(alpha), h_flip, v_flip, transform);
    }
}
