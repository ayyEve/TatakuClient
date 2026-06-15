use crate::style::*;

#[derive(Default)]
pub(super) struct LayoutData {
    pub style: Style,
    pub text_style: TextStyle,

    /// The always unrounded results of the layout computation. We must store this separately from the rounded
    /// layout to avoid errors from rounding already-rounded values. See <https://github.com/DioxusLabs/taffy/issues/501>.
    pub unrounded_layout: taffy::Layout,

    /// The final results of the layout computation.
    /// These may be rounded or unrounded depending on what the `use_rounding` config setting is set to.
    pub final_layout: taffy::Layout,

    /// The cached results of the layout computation
    pub cache: taffy::Cache,

    // /// The computation result from layout algorithm
    // pub detailed_layout_info: taffy::DetailedLayoutInfo,
}
impl LayoutData {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }
}

