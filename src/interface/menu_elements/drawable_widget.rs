use crate::prelude::*;

pub struct DrawableComponent {
    widget_sender: TripleBufferSender<Arc<dyn TatakuRenderable>>,
    event_receiver: AsyncReceiver<Bounds>,
    bounds: Bounds,

    widget: DrawableComponentWidget,
}
impl DrawableComponent {
    pub fn new() -> Self {
        let a: Arc<dyn TatakuRenderable> = Arc::new(TransformGroup::new(Vector2::ZERO));
        let (widget_sender, widget_receiver) = TripleBuffer::new(&a).split();
        let (event_sender, event_receiver) = async_channel(5);

        let widget = DrawableComponentWidget::new(widget_receiver, event_sender);
        
        Self {
            widget_sender,
            event_receiver,
            widget,
            bounds: Bounds::new(Vector2::ZERO, Vector2::ZERO)
        }
    }

    pub fn get_bounds(&mut self) -> Bounds {
        while let Ok(bounds) = self.event_receiver.try_recv() {
            self.bounds = bounds;
        }

        self.bounds
    }
    
    pub fn set_draw(&mut self, list: RenderableCollection) {
        let mut group = TransformGroup::new(Vector2::ZERO);
        list.take().into_iter().for_each(|i|group.push_arced(i));

        self.widget_sender.write(Arc::new(group));
    }

    pub fn widget(&self) -> IcedElement {
        self.widget.clone().into()
    }
}


/// this is the widget that gets added to the ui
#[derive(Clone)]
pub struct DrawableComponentWidget {
    width: iced::Length,
    height: iced::Length,

    draw_data: Arc<Mutex<TripleBufferReceiver<Arc<dyn TatakuRenderable>>>>,
    event_sender: AsyncSender<Bounds>,
}
impl DrawableComponentWidget {
    fn new(draw_data: TripleBufferReceiver<Arc<dyn TatakuRenderable>>, event_sender: AsyncSender<Bounds>) -> Self {
        Self {
            width: iced::Length::Fill,
            height: iced::Length::Fill,
            draw_data: Arc::new(Mutex::new(draw_data)),
            event_sender,
        }
    }

    pub fn width(mut self, width: iced::Length) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: iced::Length) -> Self {
        self.height = height;
        self
    }
}

impl iced::advanced::Widget<Message, IcedRenderer> for DrawableComponentWidget {
    fn width(&self) -> iced::Length { self.width }
    fn height(&self) -> iced::Length { self.height }

    fn layout(
        &self,
        _renderer: &IcedRenderer,
        limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height);

        iced_runtime::core::layout::Node::new(limits.fill())
    }

    fn draw(
        &self,
        _state: &iced_runtime::core::widget::Tree,
        renderer: &mut IcedRenderer,
        _theme: &<IcedRenderer as iced_runtime::core::Renderer>::Theme,
        _style: &iced_runtime::core::renderer::Style,
        layout: iced_runtime::core::Layout<'_>,
        _cursor: iced_runtime::core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let _ = self.event_sender.try_send(layout.bounds().into());
        renderer.draw_primitive(iced::advanced::graphics::Primitive::Custom(self.draw_data.lock().read().clone()));
    }
}

impl From<DrawableComponentWidget> for IcedElement {
    fn from(value: DrawableComponentWidget) -> Self {
        Self::new(value)
    }
}
