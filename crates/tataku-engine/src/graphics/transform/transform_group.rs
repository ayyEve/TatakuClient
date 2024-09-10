#![allow(dead_code)]
use crate::prelude::*;

#[derive(Clone)]
pub struct TransformGroup {
    pub items: Vec<Arc<dyn TatakuRenderable>>,
    transform_manager: TransformManager,

    pub scissor: Scissor,
    pub blend_mode: BlendMode,

    // pub raw_draw: bool,

    size: Vector2,
}
impl TransformGroup {
    pub fn new(pos: Vector2) -> Self {
        Self {
            items: Vec::new(),
            transform_manager: TransformManager::new(pos),

            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
            size: Vector2::ZERO,
            // raw_draw: false,
        }
    }

    pub fn from_collection(pos: Vector2, list: RenderableCollection) -> Self {
        Self {
            items: list.take(),
            transform_manager: TransformManager::new(pos),

            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
            size: Vector2::ZERO,
            // raw_draw: true,
        }
    }

    pub fn scale(mut self, scale: Vector2) -> Self {
        self.transform_manager.scale.both(scale);
        self
    }
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.transform_manager.rotation.both(rotation);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.transform_manager.alpha.both(alpha);
        self
    }
    pub fn border_alpha(mut self, alpha: f32) -> Self {
        self.transform_manager.border_alpha.both(alpha);
        self
    }
    

    pub fn recalc_size(&mut self) {
        self.size = Vector2::ZERO;
        for i in self.items.iter() {
            let b = i.get_bounds();
            self.size.x = self.size.x.max(b.pos.x + b.size.x);
            self.size.y = self.size.y.max(b.pos.y + b.size.y);
        }
    }

    pub fn push(&mut self, r: impl TatakuRenderable + 'static) {
        self.items.push(Arc::new(r));
        // self.recalc_size();
    }
    pub fn push_arced(&mut self, r: Arc<dyn TatakuRenderable>) {
        self.items.push(r);
        // self.recalc_size();
    }
}

impl TatakuRenderable for TransformGroup {
    fn get_bounds(&self) -> Bounds { 
        // for when i inevitebly forget
        error!("TransformGroup::Bounds needs work!!!!!");
        Bounds::new(self.pos.current, self.size * self.scale.current) 
    }

    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s:Scissor) { self.scissor = s; }
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode; }


    fn draw(
        &self, 
        options: &DrawOptions, 
        mut transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        let options = options.merge(DrawOptions {
            alpha: Some(self.alpha.current),
            border_alpha: Some(self.border_alpha.current),

            color: self.color.map(|c| c.current),
            border_color: None,
            // border_color: self.border_color,
        });

        transform = transform * self.transform_manager.matrix();

        self.items.iter().for_each(|i| {
            // need to scissor internal items manually
            if let Some(scissor) = i.get_scissor() {
                g.push_scissor(scissor)
            }

            i.draw(&options, transform, g);
            
            if i.get_scissor().is_some() {
                g.pop_scissor()
            }
        });
    }

    // fn draw_with_transparency(&self, _alpha: f32, _border_alpha: f32, transform: Matrix, g: &mut dyn GraphicsEngine) {
    //     self.draw(transform, g)
    // }
}


impl Deref for TransformGroup {
    type Target = TransformManager;

    fn deref(&self) -> &Self::Target {
        &self.transform_manager
    }
}
impl DerefMut for TransformGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform_manager
    }
}


#[derive(Copy, Clone)]
pub struct InitialCurrent<T> {
    pub initial: T,
    pub current: T,
}
impl<T:Clone> InitialCurrent<T> {
    pub fn new(val: T) -> Self {
        Self {
            initial: val.clone(),
            current: val,
        }
    }
    pub fn both(&mut self, val: T) {
        self.initial = val.clone();
        self.current = val;
    }
}
impl<T> Deref for InitialCurrent<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.current
    }
}
