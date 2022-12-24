#![allow(unused)]
use crate::prelude::*;

#[derive(Clone)]
pub struct SkinnedButton {
    pub pos: Vector2,
    pub size: Vector2,

    left_image: Image,
    middle_image: Image,
    right_image: Image,
}

impl SkinnedButton {
    pub async fn new(mut pos: Vector2, mut size: Vector2, depth: f64) -> Option<Self> {
        return None;

        let mut left_image = SkinManager::get_texture("button-left", true).await?;
        let mut middle_image = SkinManager::get_texture("button-middle", true).await?;
        let mut right_image = SkinManager::get_texture("button-right", true).await?;

        size += left_image.size().x() + right_image.size().x();

        let x_scale = size.x / (left_image.size.x + middle_image.size.x + right_image.size.x);

        for i in [&mut left_image, &mut middle_image, &mut right_image] {
            i.depth = depth;
            i.origin = Vector2::zero();
            i.current_color = Color::GRAY;

            i.initial_scale = Vector2::new(x_scale, size.y / i.size.y);
            i.current_scale = i.initial_scale;
        }


        Some(Self {
            pos,
            size,
            left_image,
            middle_image,
            right_image,
        })
    }

    pub fn draw(&self, _args: RenderArgs, pos_offset: Vector2, list: &mut RenderableCollection) {
        let mut current_pos = self.pos + pos_offset;

        for mut i in [
            self.left_image.clone(),
            self.middle_image.clone(),
            self.right_image.clone(),
        ] {
            i.current_pos = current_pos;
            current_pos.x += i.size().x;
            list.push(i)
        }
    }
}
