use crate::prelude::*;

#[derive(Default)]
pub struct RenderableCollection {
    pub list: Vec<Box<dyn TatakuRenderable>>,
}
impl RenderableCollection {
    pub fn push<R:TatakuRenderable + 'static>(&mut self, r: R) {
        self.list.push(Box::new(r));
    }

    pub fn take(self) -> Vec<Box<dyn TatakuRenderable>> {
        self.list
    }
}
