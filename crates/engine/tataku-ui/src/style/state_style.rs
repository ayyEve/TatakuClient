use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct ElementStateStyles<Style, T:Clone> {
    pub none: (Style, T),
    pub active: (Style, T),
    pub hover: (Style, T),
    pub focus: (Style, T),
}
impl<Style: Clone, T:Default+Clone> ElementStateStyles<Style, T> {
    pub fn new(style: Style) -> Self {
        Self {
            active: (style.clone(), T::default()),
            hover: (style.clone(), T::default()),
            focus: (style.clone(), T::default()),
            none: (style, T::default()),
        }
    }
}

impl<Style, T:Clone> ElementStateStyles<Style, T> {
    pub fn get_style(&self, state: ElementState) -> &(Style, T) {
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

    pub fn get_style_mut(&mut self, state: ElementState) -> &mut (Style, T) {
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
    
    pub fn all(&self) -> [&(Style, T); 4] {
        [
            &self.none,
            &self.active,
            &self.focus,
            &self.hover
        ]
    }
    pub fn all_mut(&mut self) -> [&mut (Style, T); 4] {
        [
            &mut self.none,
            &mut self.active,
            &mut self.focus,
            &mut self.hover
        ]
    }

    pub fn transpose<T2:Default+Clone>(self) -> ElementStateStyles<Style, T2> {
        ElementStateStyles {
            none: (self.none.0, T2::default()),
            active: (self.active.0, T2::default()),
            hover: (self.hover.0, T2::default()),
            focus: (self.focus.0, T2::default()),
        }
    }
}
