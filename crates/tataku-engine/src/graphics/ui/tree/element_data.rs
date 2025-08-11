use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Clone, Default)]
#[derive(Debug2)]
pub struct ElementData {
    pub element_name: String,
    pub id: Option<String>,
    pub class_list: Vec<String>,
    pub debug_name: Option<String>,
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
