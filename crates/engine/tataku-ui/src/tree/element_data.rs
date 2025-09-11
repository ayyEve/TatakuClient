use crate::*;
use graphics::Image;
use crate::tree::*;
use crate::style::*;

#[derive(Debug2)]
#[derive(Clone, Default)]
pub struct ElementData {
    pub element_name: ArcStr,
    pub id: Option<ArcStr>,
    pub class_list: Vec<ArcStr>,
    pub debug_name: Option<ArcStr>,
    pub state: ElementState,

    #[debug(skip)]
    pub styles: ElementStateStyles<CssStyle, Option<Image>>,
    #[debug(skip)]
    pub text_styles: ElementStateStyles<TextStyle, ()>,
}
impl ElementData {
    pub fn style(&self) -> &(CssStyle, Option<Image>) {
        self.styles.get_style(self.state)
    }
    pub fn style_mut(&mut self) -> &mut (CssStyle, Option<Image>) {
        self.styles.get_style_mut(self.state)
    }
}
