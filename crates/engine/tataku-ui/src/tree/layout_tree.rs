use crate::*;
use crate::tree::*;
use crate::style::*;
use common::reflect::*;

use taffy::Size;
use taffy::LayoutPartialTree;
use taffy::TraversePartialTree;

pub struct TaffyTreeChildIter<'a>(core::slice::Iter<'a, NodeId>);
impl Iterator for TaffyTreeChildIter<'_> {
    type Item = NodeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}

pub(super) struct LayoutTree<'a, Action: Send + Sync> {
    pub values: &'a dyn Reflect,
    pub tree: &'a mut Tree<Action>,

    pub viewport: Vector2,
    pub root_font_size: f32,

    pub use_rounding: bool,
}
impl<Action: Send + Sync> LayoutTree<'_, Action> {
    pub fn compute_layout(
        &mut self, 
        node_id: NodeId,
        available_space: Size<taffy::AvailableSpace>,
    ) {
        taffy::compute_root_layout(self, node_id, available_space);
        if self.use_rounding {
            taffy::round_layout(self, node_id);
        }
    }

    pub(crate) fn print(&mut self, root: NodeId) {
        taffy::util::print_tree(self, root);
    }
}

impl<Action: Send + Sync> taffy::CacheTree for LayoutTree<'_, Action> {
    fn cache_get(
        &self,
        node_id: NodeId,
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
        node_id: NodeId,
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

    fn cache_clear(&mut self, node_id: NodeId) {
        self.tree
            .nodes[node_id.into()]
            .cache
            .clear();
    }
}

impl<Action: Send + Sync> taffy::TraversePartialTree for LayoutTree<'_, Action> {
    type ChildIter<'b> = TaffyTreeChildIter<'b> where Self:'b;

    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        TaffyTreeChildIter(self.tree.children[parent_node_id.into()].iter())
    }

    fn child_count(&self, parent_node_id: NodeId) -> usize {
        self.tree.children[parent_node_id.into()].len()
    }

    fn get_child_id(&self, parent_node_id: NodeId, child_index: usize) -> NodeId {
        self.tree.children[parent_node_id.into()][child_index]
    }
}

impl<Action: Send + Sync> taffy::LayoutFlexboxContainer for LayoutTree<'_, Action> {
    type FlexboxContainerStyle<'b> = NodeStyleResolver<'b> where Self: 'b;
    type FlexboxItemStyle<'b> = NodeStyleResolver<'b> where Self: 'b;

    #[inline(always)]
    fn get_flexbox_container_style(&self, node_id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, child_node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl<Action: Send + Sync> taffy::LayoutBlockContainer for LayoutTree<'_, Action> {
    type BlockContainerStyle<'b> = NodeStyleResolver<'b> where Self: 'b;
    type BlockItemStyle<'b> = NodeStyleResolver<'b> where Self: 'b;

    #[inline(always)]
    fn get_block_container_style(&self, node_id: NodeId) -> Self::BlockContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    #[inline(always)]
    fn get_block_child_style(&self, child_node_id: NodeId) -> Self::BlockItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl<Action: Send + Sync> taffy::LayoutPartialTree for LayoutTree<'_, Action> {
    type CoreContainerStyle<'b> = NodeStyleResolver<'b> where Self: 'b;
    type CustomIdent = Arc<str>;

    fn get_core_container_style(
        &self, 
        node_id: NodeId
    ) -> Self::CoreContainerStyle<'_> {
        let a = self.tree.nodes.get(node_id.into()).unwrap();
        
        NodeStyleResolver {
            values: self.values,
            style: &a.style,
            viewport: self.viewport,
            root_font_size: self.root_font_size
        }
    }

    fn set_unrounded_layout(
        &mut self, 
        node_id: NodeId, 
        layout: &taffy::Layout
    ) {
        self.tree.nodes[node_id.into()].unrounded_layout = *layout;
    }

    fn compute_child_layout(
        &mut self, 
        node: NodeId, 
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
        taffy::compute_cached_layout(
            self, 
            node, 
            inputs, 
            |tree, node, inputs| 
        {
            let display_mode = tree.tree
                .nodes[node.into()]
                .style
                .get(css::CssProperty::Display)
                .and_then(|p| p.value::<DisplayType>().resolve_copied(tree.values))
                .unwrap_or_default()
                ;

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

impl<Action: Send + Sync> taffy::TraverseTree for LayoutTree<'_, Action> {}
impl<Action: Send + Sync> taffy::RoundTree for LayoutTree<'_, Action> {
    fn get_unrounded_layout(&self, node_id: NodeId) -> taffy::Layout {
        self.tree.nodes[node_id.into()].unrounded_layout
    }

    fn set_final_layout(&mut self, node_id: NodeId, layout: &taffy::Layout) {
        self.tree.nodes[node_id.into()].final_layout = *layout;
    }
}

impl<Action: Send + Sync> taffy::PrintTree for LayoutTree<'_, Action> {
    fn get_debug_label(&self, node_id: NodeId) -> &'static str {
        let ctx = self.tree.nodes.get(node_id.into()).unwrap();
        let style = &ctx.style;

        let display = style
            .get(css::CssProperty::Display)
            .and_then(|p| p.value::<DisplayType>().resolve_copied(self.values))
            ;

        let dir = style.get(css::CssProperty::FlexDirection)
            .and_then(|p| p.value::<FlexDirection>().resolve_copied(self.values))
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

    fn get_final_layout(&self, node_id: NodeId) -> taffy::Layout {
        if self.use_rounding {
            self.tree.nodes[node_id.into()].final_layout
        } else {
            self.tree.nodes[node_id.into()].unrounded_layout
        }
    }
}
