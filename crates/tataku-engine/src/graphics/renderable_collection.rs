use crate::prelude::*;

#[derive(Default)]
pub struct RenderableCollection {
    pub list: Vec<Arc<dyn TatakuRenderable>>,
    // scissors: ScissorManager
}
impl RenderableCollection {
    pub fn new() -> Self { Self::default() }

    pub fn push<R:TatakuRenderable + 'static>(&mut self, r: R) {
        // r.set_scissor(self.scissors.current_scissor());
        self.list.push(Arc::new(r));
    }

    pub fn push_scissor(&mut self, _scissor: [f32; 4]) {
        // self.scissors.push_scissor(scissor);
    }
    pub fn pop_scissor(&mut self) {
        // self.scissors.pop_scissor();
    }

    pub fn take(self) -> Vec<Arc<dyn TatakuRenderable>> {
        self.list
    }

}

