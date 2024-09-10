
use crate::prelude::*;

// pub trait StatsGraph: Send + Sync {
//     fn draw(&self, bounds: &Bounds, list: &mut RenderableCollection);
// }

use iced::advanced::Widget;

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

    pub fn view(&self) -> StatsGraphElement {
        StatsGraphElement::new(self.clone())
    }
}


pub struct StatsGraphElement {
    graph: StatsGraph,
    width: iced::Length,
    height: iced::Length,
}
impl StatsGraphElement {
    pub fn new(graph: StatsGraph) -> Self {
        Self {
            graph, 
            width: iced::Length::Fill,
            height: iced::Length::Fill,
        }
    }

    pub fn width(mut self, w: impl Into<iced::Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<iced::Length>) -> Self {
        self.height = h.into();
        self
    }
}


impl Widget<Message, iced::Theme, IcedRenderer> for StatsGraphElement {
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(self.width, self.height) }

    fn layout(
        &self,
        _tree: &mut iced_core::widget::Tree,
        _renderer: &IcedRenderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height);

        iced_core::layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _state: &iced_core::widget::Tree,
        renderer: &mut IcedRenderer,
        _theme: &iced::Theme,
        _style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds:Bounds = layout.bounds().into();
        renderer.add_renderable(self.graph.draw(&bounds));
    }
}

impl From<StatsGraphElement> for IcedElement {
    fn from(value: StatsGraphElement) -> Self {
        IcedElement::new(value)
    }
}