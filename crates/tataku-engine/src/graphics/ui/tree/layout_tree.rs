
use taffy::Size;
use crate::prelude::*;
use crate::prelude::ui::*;
use taffy::LayoutPartialTree;
use taffy::TraversePartialTree;

pub struct TaffyTreeChildIter<'a>(core::slice::Iter<'a, TaffyNodeId>);
impl Iterator for TaffyTreeChildIter<'_> {
    type Item = TaffyNodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}

pub(super) struct LayoutTree<'a> {
    pub values: &'a dyn Reflect,
    pub tree: &'a mut Tree,

    pub viewport: Vector2,
    pub root_font_size: f32,

    pub use_rounding: bool,
}
impl LayoutTree<'_> {
    pub fn compute_layout(
        &mut self, 
        node_id: taffy::NodeId,
        available_space: Size<taffy::AvailableSpace>,
    ) {
        taffy::compute_root_layout(self, node_id, available_space);
        if self.use_rounding {
            taffy::round_layout(self, node_id);
        }
    }

    pub(crate) fn print(&mut self, root: taffy::NodeId) {
        taffy::util::print_tree(self, root);
    }
}

impl taffy::CacheTree for LayoutTree<'_> {
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.tree
            .nodes[node_id.into()]
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.tree
            .nodes[node_id.into()]
            .cache
            .store(known_dimensions, available_space, run_mode, layout_output);
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.tree
            .nodes[node_id.into()]
            .cache
            .clear();
    }
}

impl taffy::TraversePartialTree for LayoutTree<'_> {
    type ChildIter<'b> = TaffyTreeChildIter<'b> where Self:'b;

    fn child_ids(&self, parent_node_id: TaffyNodeId) -> Self::ChildIter<'_> {
        TaffyTreeChildIter(self.tree.children[parent_node_id.into()].iter())
    }

    fn child_count(&self, parent_node_id: TaffyNodeId) -> usize {
        self.tree.children[parent_node_id.into()].len()
    }

    fn get_child_id(&self, parent_node_id: TaffyNodeId, child_index: usize) -> TaffyNodeId {
        self.tree.children[parent_node_id.into()][child_index]
    }
}

impl taffy::LayoutFlexboxContainer for LayoutTree<'_> {
    type FlexboxContainerStyle<'b> = CssStyleResolver<'b> where Self: 'b;
    type FlexboxItemStyle<'b> = CssStyleResolver<'b> where Self: 'b;

    #[inline(always)]
    fn get_flexbox_container_style(&self, node_id: taffy::NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl taffy::LayoutBlockContainer for LayoutTree<'_> {
    type BlockContainerStyle<'b> = CssStyleResolver<'b> where Self: 'b;
    type BlockItemStyle<'b> = CssStyleResolver<'b> where Self: 'b;

    #[inline(always)]
    fn get_block_container_style(&self, node_id: taffy::NodeId) -> Self::BlockContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    #[inline(always)]
    fn get_block_child_style(&self, child_node_id: taffy::NodeId) -> Self::BlockItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

// impl taffy::LayoutGridContainer for LayoutTree<'_> {
//     type GridContainerStyle<'b> = CssStyleResolver<'b> where Self: 'b;
//     type GridItemStyle<'b> = CssStyleResolver<'b> where Self: 'b;

//     #[inline(always)]
//     fn get_grid_container_style(&self, node_id: taffy::NodeId) -> Self::GridContainerStyle<'_> {
//         self.get_core_container_style(node_id)
//     }

//     #[inline(always)]
//     fn get_grid_child_style(&self, child_node_id: taffy::NodeId) -> Self::GridItemStyle<'_> {
//         self.get_core_container_style(child_node_id)
//     }

//     #[inline(always)]
//     fn set_detailed_grid_info(&mut self, node_id: taffy::NodeId, detailed_grid_info: taffy::DetailedGridInfo) {
//         self.tree.nodes[node_id.into()].detailed_layout_info = taffy::DetailedLayoutInfo::Grid(Box::new(detailed_grid_info));
//     }
// }


impl taffy::LayoutPartialTree for LayoutTree<'_> {
    type CoreContainerStyle<'b> = CssStyleResolver<'b> where Self: 'b;
    type CustomIdent = Arc<str>;

    fn get_core_container_style(
        &self, 
        node_id: taffy::NodeId
    ) -> Self::CoreContainerStyle<'_> {
        CssStyleResolver {
            values: self.values,
            // style: &self.tree.nodes[node_id.into()].style,
            style: &self.tree.node_context_data.get(node_id.into()).unwrap().current_style().0,
            viewport: self.viewport,
            root_font_size: self.root_font_size
        }
    }

    fn set_unrounded_layout(
        &mut self, 
        node_id: taffy::NodeId, 
        layout: &taffy::Layout
    ) {
        self.tree.nodes[node_id.into()].unrounded_layout = *layout;
    }

    fn compute_child_layout(
        &mut self, 
        node: taffy::NodeId, 
        inputs: taffy::LayoutInput
    ) -> taffy::LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == taffy::RunMode::PerformHiddenLayout {
            // debug_log!("HIDDEN");
            return taffy::compute_hidden_layout(self, node);
        }

        // We run the following wrapped in "compute_cached_layout", which will check the cache for an entry matching the node and inputs and:
        //   - Return that entry if exists
        //   - Else call the passed closure (below) to compute the result
        //
        // If there was no cache match and a new result needs to be computed then that result will be added to the cache
        taffy::compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let data = &tree.tree.nodes[node.into()];
            
            let display_mode = data
                .current_display
                .or_else(|| tree
                    .tree.get_style(node).unwrap()
                    .display
                    .resolve_copied(tree.values)
                )
                .unwrap_or_default();

            let has_children = tree.child_count(node) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (DisplayType::None, _) => taffy::compute_hidden_layout(tree, node),
                (DisplayType::Block, true) => taffy::compute_block_layout(tree, node, inputs),
                (DisplayType::Flex, true) => taffy::compute_flexbox_layout(tree, node, inputs),
                // (DisplayType::Grid, true) => todo!(), //taffy::compute_grid_layout(tree, node, inputs),
                (DisplayType::Table, true) => taffy::compute_block_layout(tree, node, inputs),
                (_, false) => {
                    let style = tree.get_core_container_style(node);
                    taffy::compute_leaf_layout(
                        inputs, 
                        &style, 
                        |_, _| 0.0, 
                        |_, _| Size::ZERO
                    )
                }
            }
        })
    }

}

impl taffy::TraverseTree for LayoutTree<'_> {}
impl taffy::RoundTree for LayoutTree<'_> {
    fn get_unrounded_layout(&self, node_id: TaffyNodeId) -> Layout {
        self.tree.nodes[node_id.into()].unrounded_layout
    }

    fn set_final_layout(&mut self, node_id: TaffyNodeId, layout: &Layout) {
        self.tree.nodes[node_id.into()].final_layout = *layout;
    }
}

impl taffy::PrintTree for LayoutTree<'_> {
    fn get_debug_label(&self, node_id: TaffyNodeId) -> &'static str {
        let node = self.tree.nodes.get(node_id.into()).unwrap();
        let style = &self
            .tree
            .node_context_data[node_id.into()]
            .current_style().0;
        let display = node
            .current_display
            .or_else(|| style.display.resolve_copied(self.values));

        let dir = style.flex_direction
            .resolve_copied(self.values)
            .unwrap_or_default();
        
        fn flex(none: bool, dir: FlexDirection) -> &'static str {
            match (none, dir) {
                (true, FlexDirection::Column) => "None? (flex, column)",
                (true, FlexDirection::ColumnReverse) => "None? (flex, column-rev)",
                (true, FlexDirection::Row) => "None? (flex, row)",
                (true, FlexDirection::RowReverse) => "None? (flex, row-rev)",

                (false, FlexDirection::Column) => "Flex-column",
                (false, FlexDirection::ColumnReverse) => "Flex-column-rev",
                (false, FlexDirection::Row) => "Flex-row",
                (false, FlexDirection::RowReverse) => "Flex-row-rev",
            }
        }

        match display {
            None => flex(true, dir),
            Some(DisplayType::Block) => "Block",
            Some(DisplayType::Flex) => flex(false, dir),
            // Some(DisplayType::Grid) => "Grid",
            Some(DisplayType::None) => "None",
            Some(DisplayType::Table) => "Table",
        }
    }

    fn get_final_layout(&self, node_id: TaffyNodeId) -> Layout {
        if self.use_rounding {
            self.tree.nodes[node_id.into()].final_layout
        } else {
            self.tree.nodes[node_id.into()].unrounded_layout
        }
    }
}
