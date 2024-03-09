use crate::prelude::*;

pub type ContentFn = Box<dyn Fn(&TransformGroup) -> IcedElement>;

pub struct TransformableComponent {
    /// we let a TransformGroup handle the transforms to avoid duplicating code
    manager: TransformManager,
}
impl TransformableComponent {
    pub fn new() -> Self {
        Self {
            manager: TransformManager::new(Vector2::ZERO),
        }
    }

    pub fn add_transform(&mut self, transform: Transformation) {
        self.manager.push_transform(transform);
    }

    pub fn update(&mut self, time: f32) {
        self.manager.update(time);
    }
    
    pub fn view(&self, content: IcedElement) -> IcedElement {
        let data = TransformableData {
            pos: self.manager.pos,
            scale: self.manager.scale,
        };
        
        TransformableContent { content, data }.into_element()
    }
}

struct TransformableData {
    pos: InitialCurrent<Vector2>,
    scale: InitialCurrent<Vector2>,
}


struct TransformableContent {
    content: IcedElement,
    data: TransformableData
}

impl iced::advanced::Widget<Message, IcedRenderer> for TransformableContent {
    fn width(&self) -> iced::Length { iced::Length::Shrink }
    fn height(&self) -> iced::Length { iced::Length::Shrink }

    fn children(&self) -> Vec<iced_runtime::core::widget::Tree> {
        vec![iced_runtime::core::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced_runtime::core::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn operate(
        &self,
        state: &mut iced_runtime::core::widget::Tree,
        layout: iced_runtime::core::Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn iced_runtime::core::widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut state.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        state: &mut iced_runtime::core::widget::Tree,
        event: iced::Event,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        renderer: &IcedRenderer,
        clipboard: &mut dyn iced_runtime::core::Clipboard,
        shell: &mut iced_runtime::core::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::event::Status {
        self.content.as_widget_mut().on_event(
            state,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport
        )
    }

    fn layout(
        &self,
        renderer: &IcedRenderer,
        limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        let offset = self.data.pos.current;
        self.content.as_widget()
            .layout(renderer, limits)
            .translate(iced::Vector::new(offset.x, offset.y))
    }

    fn draw(
        &self,
        state: &iced_runtime::core::widget::Tree,
        renderer: &mut IcedRenderer,
        theme: &<IcedRenderer as iced_runtime::core::Renderer>::Theme,
        style: &iced_runtime::core::renderer::Style,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            state, 
            renderer, 
            theme, 
            style, 
            layout, 
            cursor, 
            viewport
        )
    }

    fn mouse_interaction(
        &self,
        state: &iced_runtime::core::widget::Tree,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &IcedRenderer,
    ) -> iced_runtime::core::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            state, 
            layout, 
            cursor,
            viewport, 
            renderer
        )
    }
}

impl From<TransformableContent> for IcedElement {
    fn from(value: TransformableContent) -> Self {
        Self::new(value)
    }
}
