use crate::prelude::*;

#[derive(Default)]
pub struct RenderableCollection {
    pub list: Vec<Arc<dyn TatakuRenderable>>,
    pub do_before_add: Option<Box<dyn FnMut(&mut dyn TatakuRenderable) + Send + Sync>>,
}
impl RenderableCollection {
    pub fn new() -> Self { Self::default() }

    pub fn push<R:TatakuRenderable + 'static>(&mut self, mut r: R) {
        if let Some(do_before) = &mut self.do_before_add {
            (do_before)(&mut r);
        }
        self.list.push(Arc::new(r));
    }

    pub fn take(self) -> Vec<Arc<dyn TatakuRenderable>> {
        self.list
    }
}
