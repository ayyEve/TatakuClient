use iced::advanced::{ Overlay, Widget };
use crate::prelude::*;

pub struct DraggableDialogElement {
    dialog_content: IcedElement,
}
impl DraggableDialogElement {
    pub fn new(dialog: &Box<dyn Dialog>) -> Self {
        // use iced::widget::Tooltip;
        let view = dialog.view();

        Self {
            dialog_content: ContentBackground::new(view).rect(Some(
                Rectangle::new(
                    Vector2::ZERO, 
                    Vector2::ZERO, 
                    Color::new(0.8, 0.8, 0.8, 0.8), 
                    Some(Border::new(Color::WHITE, 2.0)))
                    .shape(Shape::Round(5.0))
                )).into_element()
        }
    }
}

impl Widget<Message, IcedRenderer> for DraggableDialogElement {
    fn width(&self) -> iced::Length { iced::Length::Fixed(0.0) }
    fn height(&self) -> iced::Length { iced::Length::Fixed(0.0) }

    fn children(&self) -> Vec<iced_runtime::core::widget::Tree> {
        vec![iced_runtime::core::widget::Tree::new(&self.dialog_content)]
    }
    fn diff(&self, tree: &mut iced_runtime::core::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.dialog_content))
    }
    fn state(&self) -> iced_runtime::core::widget::tree::State {
        iced_runtime::core::widget::tree::State::new(DraggableState::new())
    }

    fn layout(
        &self,
        _renderer: &IcedRenderer,
        _limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        iced_runtime::core::layout::Node::new(iced::Size::ZERO)
    }

    fn draw(
        &self,
        _state: &iced_runtime::core::widget::Tree,
        _renderer: &mut IcedRenderer,
        _theme: &<IcedRenderer as iced_runtime::core::Renderer>::Theme,
        _style: &iced_runtime::core::renderer::Style,
        _layout: iced_runtime::core::Layout<'_>,
        _cursor: iced_runtime::core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {}

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut iced_runtime::core::widget::Tree,
        _layout: iced_runtime::core::Layout<'_>,
        _renderer: &IcedRenderer,
    ) -> Option<iced_runtime::core::overlay::Element<'a, Message, IcedRenderer>> {
        let state2 = state.state.downcast_ref::<DraggableState>();

        Some(IcedOverlay::new(state2.pos, Box::new(DraggableOverlay {
            content: self,
            tree: state
        })))
    }
}

impl From<DraggableDialogElement> for IcedElement {
    fn from(value: DraggableDialogElement) -> Self {
        Self::new(value)
    }
}


struct DraggableOverlay<'a> {
    content: &'a mut DraggableDialogElement,
    tree: &'a mut iced_runtime::core::widget::Tree
}
impl<'a> DraggableOverlay<'a> {
    fn viewport(&self) -> iced::Rectangle {
        let state = self.tree.state.downcast_ref::<DraggableState>();
        let viewport = iced::Rectangle::new(state.pos, state.size);
        viewport
    }
}

impl<'a> Overlay<Message, IcedRenderer> for DraggableOverlay<'a> {
    fn layout(
        &self,
        renderer: &IcedRenderer,
        bounds: iced::Size,
        position: iced::Point,
    ) -> iced_runtime::core::layout::Node {
        let limits = iced_runtime::core::layout::Limits::new(iced::Size::ZERO, bounds);

        let mut node = self.content.dialog_content.as_widget().layout(renderer, &limits);
        node.move_to(position);
        node
    }
    

    fn on_event(
        &mut self,
        event: iced::Event,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        renderer: &IcedRenderer,
        clipboard: &mut dyn iced_runtime::core::Clipboard,
        shell: &mut iced_runtime::core::Shell<'_, Message>,
    ) -> iced::event::Status {
        let viewport = self.viewport();
        self.tree.state.downcast_mut::<DraggableState>().size = layout.bounds().size();

        self.content.dialog_content.as_widget_mut().on_event(
            &mut self.tree.children[0],
            event,
            layout, 
            cursor,
            renderer,
            clipboard, 
            shell, 
            &viewport
        )
    }

    fn draw(
        &self,
        renderer: &mut IcedRenderer,
        theme: &<IcedRenderer as iced_runtime::core::Renderer>::Theme,
        style: &iced_runtime::core::renderer::Style,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
    ) {
        let viewport = self.viewport();
        self.content.dialog_content.as_widget().draw(
            &self.tree.children[0],
            renderer, 
            theme,
            style, 
            layout,
            cursor, 
            &viewport
        )
    }


    fn is_over(
        &self,
        layout: iced_runtime::core::Layout<'_>,
        _renderer: &IcedRenderer,
        cursor_position: iced::Point,
    ) -> bool {
        layout.bounds().contains(cursor_position)
    }

    fn mouse_interaction(
        &self,
        layout: iced_runtime::core::Layout<'_>,
        cursor: iced_runtime::core::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &IcedRenderer,
    ) -> iced_runtime::core::mouse::Interaction {
        self.content.dialog_content.as_widget().mouse_interaction(
            &self.tree.children[0], 
            layout, 
            cursor, 
            viewport, 
            renderer
        )
    }

    fn operate(
        &mut self,
        layout: iced_runtime::core::Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn iced_runtime::core::widget::Operation<Message>,
    ) {
        self.content.dialog_content.as_widget_mut().operate(
            &mut self.tree.children[0], 
            layout, 
            renderer, 
            operation
        )
    }
}

struct DraggableState {
    pos: iced::Point,
    size: iced::Size,
}
impl DraggableState {
    fn new() -> Self {
        Self {
            pos: iced::Point::ORIGIN,
            size: iced::Size::ZERO
        }
    }
}