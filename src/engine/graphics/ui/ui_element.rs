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
    pub async fn new<T:'static+InnerUIElement>(name: &str, default_pos: Vector2, inner: T) -> Self {
        let element_name = name.to_owned();
        let mut pos_offset = default_pos;
        let mut scale = Vector2::one();
        let mut visible = true;
        
        if let Some((stored_pos, stored_scale, stored_window_size, stored_visible)) = Database::get_element_info(&element_name).await {
            pos_offset = stored_pos;
            scale = stored_scale;
            visible = stored_visible;
            
            if stored_window_size.length() > 0.0 {
                // debug!("got stored window size {stored_window_size:?}");
                do_scale(&mut pos_offset, &mut scale, stored_window_size, WindowSize::get().0);
            }

        }

        if scale.x.abs() < 0.01 { scale.x = 1.0 }
        if scale.y.abs() < 0.01 { scale.y = 1.0 }

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
        if !self.visible { return }
        self.inner.update(manager);
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        if !self.visible { return }
        self.inner.draw(self.pos_offset, self.scale, list);
    }

    pub fn get_bounds(&self) -> Rectangle {
        let mut base = self.inner.get_bounds();
        base.pos += self.pos_offset;
        base.size *= self.scale;
        base
    }

    pub async fn save(&self) {
        Database::save_element_info(self.pos_offset, self.scale, self.visible, &self.element_name).await;
    }

    pub async fn clear_save(&self) {
        Database::clear_element_info(&self.element_name).await;
    }

    pub fn reset_element(&mut self) {
        self.inner.reset();
    }
}

pub trait InnerUIElement: Send + Sync {
    fn update(&mut self, manager: &mut IngameManager);
    fn draw(&mut self, pos_offset: Vector2, scale: Vector2, list: &mut RenderableCollection);
    fn get_bounds(&self) -> Rectangle;
    fn reset(&mut self) {}
    fn display_name(&self) -> &'static str;
}

#[allow(unused)]
fn do_scale(pos: &mut Vector2, scale: &mut Vector2, old_window_size: Vector2, new_window_size: Vector2) {
    // TODO:
    // let new_scale = new_window_size / old_window_size;
    // let scaled_pos_offset = new_window_size - old_window_size * new_scale;

    // *pos = scaled_pos_offset + *pos * new_scale;
    // *scale *= new_scale
}