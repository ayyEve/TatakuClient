use crate::prelude::*;

#[derive(Clone, Default2)]
pub struct TreeData {
    // pub bounds: Bounds,
    pub absolute_bounds: Bounds,
    pub local_transform: Transform,

    #[default(Matrix::identity())]
    pub global_transform: Matrix,
    
    #[default(Matrix::identity())]
    pub inverse_global_transform: Matrix,

    pub needs_inverse_transform: bool,

    pub selected: Option<bool>,
    adjacent_nodes: [Option<taffy::NodeId>; 4],

    pub element_data: ElementData,
}
impl TreeData {
    pub fn with_style(style: CssStyle) -> Self {
        Self {
            element_data: ElementData { 
                styles: ElementStateStyles::new(style.clone()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn selectable(&self) -> bool {
        self.selected.is_some()
    }
    pub fn set_selectable(&mut self, selectable: bool) {
        self.selected = selectable.then_some(false);
    }

    pub fn node_direction(&self, direction: Direction) -> Option<taffy::NodeId> {
        self.adjacent_nodes[direction as u8 as usize]
        // match direction {
        //     Direction::Up => self.node_above,
        //     Direction::Down => self.node_below,
        //     Direction::Left => self.node_left,
        //     Direction::Right => self.node_right,
        // }
    }
    pub fn set_node_direction(&mut self, direction: Direction, node: Option<taffy::NodeId>) {
        self.adjacent_nodes[direction as u8 as usize] = node;
        // match direction {
        //     Direction::Up => self.node_above,
        //     Direction::Down => self.node_below,
        //     Direction::Left => self.node_left,
        //     Direction::Right => self.node_right,
        // }
    }


    pub fn set_styles<_T:Clone>(
        &mut self, 
        styles: ElementStateStyles<CssStyle, _T>, 
        values: &dyn Reflect
    ) {
        self.element_data.styles = styles.transpose();

        for i in ElementState::list() {
            let txt = self.element_data.styles.get_style(*i).0.text_style(values);
            let (s, _) = self.element_data.text_styles.get_style_mut(*i);
            *s = txt;
        }
    }

    pub fn get_style(&self, state: ElementState) -> &CssStyle {
        &self.element_data
            .styles
            .get_style(state).0
    }

    pub fn current_style(&self) -> &(CssStyle, Option<Image>) {
        self.element_data.style()
    }
    pub fn current_text_style(&self) -> &TextStyle {
        &self.element_data
            .text_styles
            .get_style(self.element_data.state)
            .0
    }
}
