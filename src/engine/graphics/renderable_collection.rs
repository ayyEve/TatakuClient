use crate::prelude::*;

#[derive(Default)]
pub struct RenderableCollection {
    pub list: Vec<Arc<dyn TatakuRenderable>>,
    pub do_before_add: Option<Box<dyn FnMut(&mut dyn TatakuRenderable) + Send + Sync>>,

    scissors: ScissorManager
}
impl RenderableCollection {
    pub fn new() -> Self { Self::default() }

    pub fn push<R:TatakuRenderable + 'static>(&mut self, mut r: R) {
        if let Some(do_before) = &mut self.do_before_add {
            (do_before)(&mut r);
        }

        r.set_scissor(self.scissors.current_scissor());
        self.list.push(Arc::new(r));
    }

    pub fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push_scissor(scissor);
    }
    pub fn pop_scissor(&mut self) {
        self.scissors.pop_scissor();
    }

    pub fn take(self) -> Vec<Arc<dyn TatakuRenderable>> {
        self.list
    }

}


#[derive(Default)]
pub struct ScissorManager {
    scissors: Vec<[f32; 4]>,
    current_scissor: Scissor
}
impl ScissorManager {
    pub fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push(scissor);
        self.recalc_current_scissor();
    }
    pub fn pop_scissor(&mut self) {
        self.scissors.pop();
        self.recalc_current_scissor();
    }

    pub fn current_scissor(&self) -> Scissor {
        self.current_scissor
    }

    fn recalc_current_scissor(&mut self) {
        if self.scissors.is_empty() {
            self.current_scissor = None;
            return;
        }

        let s = self.scissors.iter().fold([f32::MIN, f32::MIN, f32::MAX, f32::MAX], |i, n| {
            [
                i[0].max(n[0]),
                i[1].max(n[1]),
                i[2].min(n[2]),
                i[3].min(n[3]),
            ]
        });
        self.current_scissor = Some(s);
    }
}