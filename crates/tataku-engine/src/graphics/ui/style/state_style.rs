use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Clone, Debug, Default)]
pub struct ElementStateStyles<T:Clone> {
    pub none: (CssStyle, T),
    pub active: (CssStyle, T),
    pub hover: (CssStyle, T),
    pub focus: (CssStyle, T),
}
impl<T:Default+Clone> ElementStateStyles<T> {
    pub fn new(
        none: CssStyle,
        active: CssStyle,
        hover: CssStyle,
        focus: CssStyle,
    ) -> Self {
        Self {
            active: (active.merge(none.clone()), T::default()),
            hover: (hover.merge(none.clone()), T::default()),
            focus: (focus.merge(none.clone()), T::default()),
            none: (none, T::default()),
        }
    }
}

impl<T:Clone> ElementStateStyles<T> {
    pub fn get_style(&self, state: ElementState) -> &(CssStyle, T) {
        if state.contains(ElementState::Active) {
            &self.active
        } else if state.contains(ElementState::Hover) {
            &self.hover
        } else if state.contains(ElementState::Focus) {
            &self.focus
        } else {
            &self.none
        }
    }

    pub fn get_style_mut(&mut self, state: ElementState) -> &mut (CssStyle, T) {
        if state.contains(ElementState::Active) {
            &mut self.active
        } else if state.contains(ElementState::Hover) {
            &mut self.hover
        } else if state.contains(ElementState::Focus) {
            &mut self.focus
        } else {
            &mut self.none
        }
    }
    
    pub fn all(&self) -> [&(CssStyle, T); 4] {
        [
            &self.none,
            &self.active,
            &self.focus,
            &self.hover
        ]
    }
    pub fn all_mut(&mut self) -> [&mut (CssStyle, T); 4] {
        [
            &mut self.none,
            &mut self.active,
            &mut self.focus,
            &mut self.hover
        ]
    }

    pub fn transpose<T2:Default+Clone>(self) -> ElementStateStyles<T2> {
        ElementStateStyles {
            none: (self.none.0, T2::default()),
            active: (self.active.0, T2::default()),
            hover: (self.hover.0, T2::default()),
            focus: (self.focus.0, T2::default()),
        }
    }
}
