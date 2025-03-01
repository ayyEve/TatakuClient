use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Clone)]
pub enum StatsGraph {
    Bar(Box<BarGraph>),
    Pie(Box<PieGraph>),
    Scatter(Box<ScatterGraph>),
}
impl StatsGraph {
    pub fn draw(&self, bounds: &Bounds) -> Arc<dyn TatakuRenderable> {
        let group = match self {
            Self::Bar(bar) => bar.draw(bounds),
            Self::Pie(pie) => pie.draw(bounds),
            Self::Scatter(scatter) => scatter.draw(bounds),
        };

        Arc::new(group)
    }

    pub fn view(&self) -> StatsGraphWidget {
        StatsGraphWidget::new(self.clone())
    }
}

pub struct StatsGraphWidget {
    graph: StatsGraph,
    style: Style,
    node_id: NodeId,
}
impl StatsGraphWidget {
    pub fn new(graph: StatsGraph) -> Self {
        Self {
            graph, 
            style: Style {
                size: Size {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Percent(1.0),
                },
                ..Default::default()
            },
            node_id: Default::default(),
        }
    }
}

impl Widget for StatsGraphWidget {
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
        shell.list.push_arced(self.graph.draw(&bounds));
    }
}
