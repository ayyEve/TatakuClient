use crate::prelude::*;

pub struct UIElement {
    pub element_name: String,
    pub pos_offset: Vector2,
    pub scale: Vector2,

    pub inner: Box<dyn InnerUIElement>
}

impl UIElement {
    pub fn new<T:'static+InnerUIElement>(name: &str, default_pos: Vector2, inner: T) -> Self {
        // TODO: get the saved values

        Self {
            element_name: name.to_owned(),
            pos_offset: default_pos,
            scale: Vector2::one(),
            inner: Box::new(inner)
        }
    }

    pub fn update(&mut self, manager: &mut IngameManager) {
        self.inner.update(manager);
    }

    pub fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        self.inner.draw(self.pos_offset, self.scale, list)
    }
}

pub trait InnerUIElement {
    fn update(&mut self, manager: &mut IngameManager);
    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut Vec<Box<dyn Renderable>>);
}