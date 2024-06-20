use iced::advanced::{ Overlay, Widget };
use crate::prelude::*;

pub struct DraggableDialogElement {
    dialog_content: IcedElement,
}
impl DraggableDialogElement {
    pub fn new(dialog: &Box<dyn Dialog>, values: &mut ValueCollection) -> Self {
        // use iced::widget::Tooltip;
        let view = dialog.view(values);

        Self {
            dialog_content: ContentBackground::new(view)
                .color(Some(Color::new(0.8, 0.8, 0.8, 0.8)))
                .border(Some(Border::new(Color::WHITE, 2.0)))
                .shape(Shape::Round(5.0))
                .into_element()
        }
    }
}

impl Widget<Message, iced::Theme, IcedRenderer> for DraggableDialogElement {    
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(iced::Length::Fixed(0.0), iced::Length::Fixed(0.0)) }


    fn children(&self) -> Vec<iced_core::widget::Tree> {
        vec![iced_core::widget::Tree::new(&self.dialog_content)]
    }
    fn diff(&self, tree: &mut iced_core::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.dialog_content))
    }
    fn state(&self) -> iced_core::widget::tree::State {
        iced_core::widget::tree::State::new(DraggableState::new())
    }

    fn layout(
        &self,
        _state: &mut iced_core::widget::Tree,
        _renderer: &IcedRenderer,
        _limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        iced_core::layout::Node::new(iced::Size::ZERO)
    }

    fn draw(
        &self,
        _state: &iced_core::widget::Tree,
        _renderer: &mut IcedRenderer,
        _theme: &iced::Theme,
        _style: &iced_core::renderer::Style,
        _layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {}

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut iced_core::widget::Tree,
        _layout: iced_core::Layout<'_>,
        _renderer: &IcedRenderer,
        offset: iced::Vector,
    ) -> Option<iced_core::overlay::Element<'a, Message, iced::Theme, IcedRenderer>> {
        let state2 = state.state.downcast_ref::<DraggableState>();

        Some(IcedOverlay::new(Box::new(DraggableOverlay {
            content: self,
            tree: state,
            offset
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
    tree: &'a mut iced_core::widget::Tree,
    offset: iced::Vector
}
impl<'a> DraggableOverlay<'a> {
    fn viewport(&self) -> iced::Rectangle {
        let state = self.tree.state.downcast_ref::<DraggableState>();
        let viewport = iced::Rectangle::new(state.pos, state.size);
        viewport
    }
}

impl<'a> Overlay<Message, iced::Theme, IcedRenderer> for DraggableOverlay<'a> {
    fn layout(
        &mut self,
        renderer: &IcedRenderer,
        size: iced::Size,
    ) -> iced_core::layout::Node {
        let limits = iced_core::layout::Limits::new(iced::Size::ZERO, size);

        let mut node = self.content.dialog_content.as_widget().layout(self.tree, renderer, &limits);
        node.move_to(iced::Point::new(self.offset.x, self.offset.y))
    }
    

    fn on_event(
        &mut self,
        event: iced::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &IcedRenderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
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
        theme: &iced::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
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
        layout: iced_core::Layout<'_>,
        _renderer: &IcedRenderer,
        cursor_position: iced::Point,
    ) -> bool {
        layout.bounds().contains(cursor_position)
    }

    fn mouse_interaction(
        &self,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &IcedRenderer,
    ) -> iced_core::mouse::Interaction {
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
        layout: iced_core::Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn iced_core::widget::Operation<Message>,
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