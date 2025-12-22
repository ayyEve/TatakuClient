use crate::prelude::*;
use widgets::*;
use ui::{
    tree::*,
    widget::*,
};
use engine::gameplay::stats::{
    GraphType,
    StatsInfo,
    StatsEntry,
};

pub struct StatsGraphWidget {
    node: Box<dyn Widget<actions::Action>>,
    node_id: NodeId
}
impl StatsGraphWidget {
    pub fn new(stats: &StatsInfo) -> Self {
        let node = Self::view(stats);

        Self {
            // stats,
            node,
            node_id: ui::EMPTY_NODE,
        }
    }

    fn view(_stats: &StatsInfo) -> Box<dyn Widget<actions::Action>> {
        EmptyWidget::new_boxed()
        // Container::new(
        //     vec![
        //         // display name should be at the top (TODO: with some margin above and below )
        //         TextWidget::new(stats.display_name.clone())
        //             .font_size(30.0)
        //             .text_color(Color::BLACK)
        //             .width(FILL)
        //             // .margin()
        //             .boxed(),

        //         // ~half the remaining vertical should be for listing the values
        //         Container::new(stats.data.iter()
        //             .filter(|i| i.show_in_list)
        //             .map(|i| TextWidget::new(format!(
        //                     "{}: {}",
        //                     i.name,
        //                     format_float(i.get_value(), 2)
        //                 ))
        //                 .font_size(20.0)
        //                 .text_color(i.color)
        //                 .width(FILL)
        //                 .boxed()
        //             ).collect::<Vec<_>>()
        //         )
        //         .flex_direction(FlexDirection::Column)
        //         .height(CssUnit::Auto)
        //         .boxed(),

        //         // the remaining space should be used for the graph
        //         GraphWidget::new(stats.graph_type, &stats.data)
        //             .width(FILL)
        //             // .height(Dimension::percent(0.4))
        //             // .margin([4.0, 0.0, 0.0, 0.0])
        //             .boxed(),
        //     ]
        // )
        // .flex_direction(FlexDirection::Column)
        // .width(FILL)
        // .height(FILL)
        // .boxed()
    }
}
impl Widget<actions::Action> for StatsGraphWidget {
    fn name(&self) -> CowStr { "stats_graph_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::Single(&*self.node)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::Single(&mut *self.node)
    }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<actions::Action>,
    ) -> taffy::TaffyResult<NodeId> {
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(&[ child ])?;

        Ok(self.node_id)
    }
}


pub struct GraphWidget {
    graph: StatsGraph,
    node_id: NodeId,
}
impl GraphWidget {
    pub fn new(graph_type: GraphType, data: &Arc<Vec<StatsEntry>>) -> Self {
        let graph = match graph_type {
            GraphType::Pie => StatsGraph::Pie(Box::new(PieGraph::new(data.clone()))),
            GraphType::Bar => StatsGraph::Bar(Box::new(BarGraph::new(data.clone()))),
            GraphType::Scatter => StatsGraph::Scatter(Box::new(ScatterGraph::new(data.clone()))),
        };

        // let one = half::f16::from_f32(1.0);
        Self {
            graph,
            // style: CssStyle {
            //     width: CssUnit::Percent(one).into(),
            //     height: CssUnit::Percent(one).into(),
            //     ..Default::default()
            // },
            node_id: ui::EMPTY_NODE,
        }
    }
}
impl Widget<actions::Action> for GraphWidget {
    fn name(&self) -> CowStr { "graph_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<actions::Action>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        let collection = match &self.graph {
            StatsGraph::Bar(bar) => bar.draw(&bounds),
            StatsGraph::Pie(pie) => pie.draw(&bounds),
            StatsGraph::Scatter(scatter) => scatter.draw(&bounds),
        };

        shell.list.list.extend(collection.list);
    }
}



#[derive(Clone)]
enum StatsGraph {
    Bar(Box<BarGraph>),
    Pie(Box<PieGraph>),
    Scatter(Box<ScatterGraph>),
}
