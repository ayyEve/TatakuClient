use crate::prelude::*;

pub struct UIElement {
    pub default_pos: Vector2,
    pub element_name: String,
    pub pos_offset: Vector2,
    pub scale: Vector2,
    pub visible: bool,

    pub inner: Box<dyn InnerUIElement>
}

impl UIElement {
    pub fn new<T:'static+InnerUIElement>(name: &str, default_pos: Vector2, inner: T) -> Self {
        let element_name = name.to_owned();
        let mut pos_offset = default_pos;
        let mut scale = Vector2::one();
        let mut visible = true;
        
        if let Some((pos, scale2, visible2)) = Database::get_info(&element_name) {
            pos_offset = pos;
            scale = scale2;
            visible = visible2;
        }

        if scale.x.abs() < 0.01 {scale.x = 1.0}
        if scale.y.abs() < 0.01 {scale.y = 1.0}

        Self {
            default_pos,
            element_name,
            pos_offset,
            scale,
            inner: Box::new(inner),
            visible
        }
    }

    pub fn update(&mut self, manager: &mut IngameManager) {
        if !self.visible {return}
        self.inner.update(manager);
    }

    pub fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}
        self.inner.draw(self.pos_offset, self.scale, list)
    }

    pub fn get_bounds(&self) -> Rectangle {
        let mut base = self.inner.get_bounds();
        base.current_pos += self.pos_offset;
        base.size *= self.scale;
        base
    }
}

pub trait InnerUIElement {
    fn update(&mut self, manager: &mut IngameManager);
    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>);
    fn get_bounds(&self) -> Rectangle;
}