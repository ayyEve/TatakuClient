#![allow(unused)]
use crate::prelude::*;

#[derive(Clone)]
pub struct GenericButtonImage {
    pub pos: Vector2,
    pub size: Vector2,
    pub color: Color,

    left_image: Image,
    middle_image: Image,
    right_image: Image,
}

impl GenericButtonImage {
    pub async fn new(pos: Vector2, size: Vector2) -> Option<Self> {
        // return None;
        let mut left_image = SkinManager::get_texture("button-left", true).await?;
        left_image.origin = Vector2::ZERO;
        
        let mut right_image = SkinManager::get_texture("button-right", true).await?;
        right_image.origin = Vector2::ZERO;

        let mut middle_image = SkinManager::get_texture("button-middle", true).await?;
        middle_image.origin = Vector2::ZERO;

        let mut s = Self {
            pos,
            size,
            color: Color::WHITE,
            left_image,
            middle_image,
            right_image,
        };
        s.set_size(size);

        Some(s)
    }

    pub fn set_size(&mut self, size: Vector2) {
        self.left_image.scale = Vector2::ONE * size.y / self.left_image.tex_size().y;
        self.right_image.scale = Vector2::ONE * size.y / self.right_image.tex_size().y;

        let w1 = self.left_image.size().x;
        let w3 = self.right_image.size().x;
        self.right_image.pos = Vector2::with_x(size.x - w3);

        self.middle_image.pos = Vector2::with_x(w1);
        self.middle_image.set_size(Vector2::new(
            size.x - (w1 + w3),
            size.y
        ));
    }

    pub fn draw(&self, _args: RenderArgs, depth: f64, pos_offset: Vector2, list: &mut RenderableCollection) {
        let current_pos = self.pos + pos_offset;

        for mut i in [
            self.left_image.clone(),
            self.middle_image.clone(),
            self.right_image.clone(),
        ] {
            i.pos += current_pos;
            i.depth = depth;
            i.color = self.color;
            list.push(i)
        }
    }
}
