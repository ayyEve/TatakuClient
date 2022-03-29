use crate::prelude::*;

pub struct UIElement {
    pub element_name: String,
    pub pos_offset: Vector2,
    pub scale: Vector2,

    pub inner: Box<dyn InnerUIElement>
}

impl UIElement {
    pub fn new<T:'static+InnerUIElement>(name: &str, default_pos: Vector2, inner: T) -> Self {
        let element_name = name.to_owned();
        let mut pos_offset = default_pos;
        let mut scale = Vector2::one();
        
        if let Some((pos, scale2)) = Database::get_info(&element_name) {
            pos_offset = pos;
            scale = scale2;
        }

        Self {
            element_name,
            pos_offset,
            scale,
            inner: Box::new(inner)
        }
    }

    pub fn update(&mut self, manager: &mut IngameManager) {
        self.inner.update(manager);
    }

    pub fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        self.inner.draw(self.pos_offset, self.scale, list)
    }

    pub fn get_bounds(&self) -> Rectangle {
        let mut base = self.inner.get_bounds();
        base.current_pos += self.pos_offset;
        base.size *= self.scale;

        info!("{}: {:?}, {:?}", self.element_name, base.current_pos, base.size);
        base
    }
}

pub trait InnerUIElement {
    fn update(&mut self, manager: &mut IngameManager);
    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>);
    fn get_bounds(&self) -> Rectangle;
}