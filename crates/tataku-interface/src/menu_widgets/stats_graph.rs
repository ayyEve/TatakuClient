use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Widget)]
pub struct StatsGraphWidget {
    style: Style,
    node: Box<dyn Widget>,
    node_id: NodeId
}
impl StatsGraphWidget {
    pub fn new(stats: &StatsInfo) -> Self {
        let node = Self::view(stats);

        Self {
            // stats,

            style: Style::default(),
            node,
            node_id: NodeId::default(),
        }
    }

    fn view(stats: &StatsInfo) -> Box<dyn Widget> {
        Container::new(
            vec![
                // display name should be at the top (TODO: with some margin above and below )
                TextWidget::new(stats.display_name.clone())
                    .font_size(30.0)
                    .text_color(Color::BLACK)
                    .width(FILL)
                    // .margin()
                    .boxed(),

                // ~half the remaining vertical should be for listing the values 
                Container::new(stats.data.iter()
                    .filter(|i| i.show_in_list)
                    .map(|i| TextWidget::new(format!(
                            "{}: {}", 
                            i.name, 
                            format_float(i.get_value(), 2)
                        ))
                        .font_size(20.0)
                        .text_color(i.color)
                        .width(FILL)
                        .boxed()
                    ).collect::<Vec<_>>()
                )
                .flex_direction(FlexDirection::Column)
                .height(Dimension::Auto)
                .boxed(),
                
                // the remaining space should be used for the graph
                GraphWidget::new(stats.graph_type, &stats.data)
                    .width(FILL)
                    // .height(Dimension::Percent(0.4))
                    // .margin([4.0, 0.0, 0.0, 0.0])
                    .boxed(),
            ]
        )
        .flex_direction(FlexDirection::Column)
        .width(Dimension::Percent(1.0))
        .height(Dimension::Percent(1.0))
        .boxed()
    }
}
impl Widget for StatsGraphWidget {
    fn name(&self) -> Cow<'static, str> { "stats_graph_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell, 
        display_override: Option<ui::Display>
    ) {
        self.node.update_styles(shell, display_override);
    }

    fn layout(
        &mut self,
        shell: &mut LayoutShell<'_>
    ) -> taffy::TaffyResult<NodeId> {
        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            self.style.clone(),
            &[ child ]
        )?;

        Ok(self.node_id)
    }
}


#[derive(Widget)]
pub struct GraphWidget {
    graph: StatsGraph,
    style: Style,
    node_id: NodeId,
}
impl GraphWidget {
    pub fn new(graph_type: GraphType, data: &Arc<Vec<StatsEntry>>) -> Self {
        let graph = match graph_type {
            GraphType::Pie => StatsGraph::Pie(Box::new(PieGraph::new(data.clone()))),
            GraphType::Bar => StatsGraph::Bar(Box::new(BarGraph::new(data.clone()))),
            GraphType::Scatter => StatsGraph::Scatter(Box::new(ScatterGraph::new(data.clone()))),
        };

        Self {
            graph, 
            style: Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            },
            node_id: NodeId::default(),
        }
    }
}
impl Widget for GraphWidget {
    fn name(&self) -> Cow<'static, str> { "stats_graph_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }
    
    fn layout(
        &mut self, 
        shell: &mut LayoutShell<'_>
    ) -> TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf(self.style.clone())?;
        Ok(self.node_id)
    }
    
    fn draw(&self, shell: &mut DrawShell<'_>) {
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) else { return };

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
