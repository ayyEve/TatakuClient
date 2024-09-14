use crate::prelude::*;


#[derive(Clone, Default)]
pub struct ScissorGroup {
    pub items: Vec<Arc<dyn TatakuRenderable>>,
    pub scissor: Scissor,
    size: Vector2,
}
impl ScissorGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_collection(list: RenderableCollection) -> Self {
        Self {
            items: list.take(),

            scissor: None,
            size: Vector2::ZERO,
        }
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

impl TatakuRenderable for ScissorGroup {
    fn get_bounds(&self) -> Bounds { 
        // for when i inevitebly forget
        self.scissor.map(Bounds::from).unwrap_or_default()
    }

    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s:Scissor) { self.scissor = s; }
    fn get_blend_mode(&self) -> BlendMode { BlendMode::None }
    fn set_blend_mode(&mut self, _blend_mode: BlendMode) { }


    // fn draw(&self, transform: Matrix, g: &mut dyn GraphicsEngine) {
    //     self.items.iter().for_each(|i| {
    //         // need to scissor internal items manually
    //         if let Some(scissor) = i.get_scissor() {
    //             g.push_scissor(scissor)
    //         }

    //         i.draw(transform, g);

    //         if i.get_scissor().is_some() {
    //             g.pop_scissor()
    //         }
    //     });
    // }

    fn draw(&self, options: &DrawOptions, transform: Matrix, g: &mut dyn GraphicsEngine) {
        self.items.iter().for_each(|i| {
            // need to scissor internal items manually
            if let Some(scissor) = i.get_scissor() {
                g.push_scissor(scissor)
            }

            i.draw(options, transform, g);
            
            if i.get_scissor().is_some() {
                g.pop_scissor()
            }
        });
    }
}
