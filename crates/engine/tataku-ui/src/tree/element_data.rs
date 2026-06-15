use crate::*;
use crate::tree::*;

#[derive(Debug2)]
#[derive(Clone, Default)]
pub struct ElementData {
    pub element_name: ArcStr,
    pub default_style: ArcStr,
    pub id: Option<ArcStr>,
    pub class_list: Vec<ArcStr>,
    pub debug_name: Option<ArcStr>,
    pub state: ElementState,
}