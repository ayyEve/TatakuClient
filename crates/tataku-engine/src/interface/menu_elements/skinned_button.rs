use crate::prelude::*;

use iced::Theme;
use iced_core::{
    keyboard,
    layout,
    mouse,
    overlay,
    renderer,
    touch,
    event::{self, Event},
    widget::operation::Operation,
    widget::tree::{self, Tree},
    
    Clipboard, Layout, Length, Point,
    Rectangle, Shell, Padding
};

pub struct SkinnedButton {
    content: IcedElement,
    content_fn: Box<dyn Fn(&SkinParams) -> IcedElement + 'static>,

    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    // style: <Renderer::Theme as StyleSheet>::Style,

    image: Option<Image>,
    is_selected: bool,
    is_hover: bool,

    base_params: SkinParams,
    hover_params: SkinParams,
    selected_params: SkinParams,

    /// if enter is pressed while we're selected, do we send our message?
    handle_enter_keypress: bool,
}

impl SkinnedButton {

    /// Creates a new [`Button`] with the given content.
    pub fn new(content_fn: impl Fn(&SkinParams) -> IcedElement + 'static) -> Self {
        let base_params = SkinParams::default();
        let content = (content_fn)(&base_params); //iced::widget::Text::new(text.clone()).size(font_size).width(Length::Fill).into_element();

        Self {
            content,
            content_fn: Box::new(content_fn),

            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            // style: <Renderer::Theme as StyleSheet>::Style::default(),

            image: None,
            is_selected: false,
            is_hover: false,

            
            base_params,
            hover_params: SkinParams::default(),
            selected_params: SkinParams::default(),

            handle_enter_keypress: false,
        }
    }

    /// Sets the width of the [`Button`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Button`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the [`Button`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Button`] will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }



    /// set this button's image
    pub fn image(mut self, mut image: Option<Image>) -> Self {
        image.as_mut().map(|i| i.origin = Vector2::ZERO);
        self.image = image;
        self
    }

    /// Set if this button is selected
    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn base_params(mut self, params: SkinParams) -> Self {
        self.base_params = params;
        self
    }
    pub fn selected_params(mut self, params: SkinParams) -> Self {
        self.selected_params = params;
        self
    }
    pub fn hover_params(mut self, params: SkinParams) -> Self {
        self.hover_params = params;
        self
    }

    pub fn handle_enter_keypress(mut self, handle: bool) -> Self {
        self.handle_enter_keypress = handle;
        self
    }

    pub fn build(mut self) -> Self {
        self.update_content();
        self
    }


    
    fn get_skin_params(&self) -> SkinParams {
        if self.is_hover {
            self.hover_params
        } else if self.is_selected {
            self.selected_params
        } else {
            self.base_params
        }
    }

    fn update_content(&mut self) {
        let skin_params = self.get_skin_params();
        self.content = (self.content_fn)(&skin_params);
    }

}


impl iced::advanced::Widget<Message, iced::Theme, IcedRenderer> for SkinnedButton {
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(self.width, self.height) }
    fn tag(&self) -> tree::Tag { tree::Tag::of::<State>() }

    fn state(&self) -> tree::State { tree::State::new(State::default()) }
    fn children(&self) -> Vec<Tree> { vec![Tree::new(&self.content)] }
    fn diff(&self, tree: &mut Tree) { tree.diff_children(std::slice::from_ref(&self.content)) }


    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &IcedRenderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let skin_params = self.get_skin_params();
        let limits = limits.width(self.width).height(self.height);

        let content = self.content.as_widget().layout(&mut tree.children[0], renderer, &limits);
        let padding = self.padding.fit(content.size(), limits.max());
        let size = limits.resolve(self.width, self.height, content.size());
    
        let content = content.move_to(Point::new(padding.left + skin_params.offset.x, padding.top + skin_params.offset.y));
    
        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &IcedRenderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();
        let old_hover = self.is_hover;
        self.is_hover = cursor.is_over(bounds);
        if self.is_hover != old_hover {
            self.update_content();
        }

        // if let event::Status::Captured = self.content.as_widget_mut().on_event(
        //     &mut tree.children[0],
        //     event.clone(),
        //     layout.children().next().unwrap(),
        //     cursor,
        //     renderer,
        //     clipboard,
        //     shell,
        //     viewport,
        // ) {
        //     return event::Status::Captured;
        // }

        let state = tree.state.downcast_mut::<State>();
        match event {
            // pressed
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => if self.on_press.is_some() && self.is_hover {
                state.is_pressed = true;
                return event::Status::Captured;
            }
            // released
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) if state.is_pressed => {
                    if let Some(on_press) = self.on_press.clone() {
                    state.is_pressed = false;

                    if self.is_hover {
                        shell.publish(on_press);
                    }

                    return event::Status::Captured;
                }
            }

            Event::Keyboard(keyboard::Event::KeyPressed { key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter), .. }) if self.handle_enter_keypress && self.is_selected => {
                if let Some(on_press) = self.on_press.clone() {
                    shell.publish(on_press);
                    return event::Status::Captured;
                }
            }

            Event::Touch(touch::Event::FingerLost { .. }) => {
                state.is_pressed = false;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut IcedRenderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        let params = self.get_skin_params();

        if let Some(mut image) = self.image.clone() {
            image.pos = bounds.position().into();
            image.set_size(bounds.size().into());
            image.color = params.background;

            renderer.add_renderable(Arc::new(image));
        } else {
            renderer.add_renderable(Arc::new(
                crate::prelude::Rectangle::new(
                    bounds.position().into(),
                    bounds.size().into(),
                    Color::new(0.2, 0.2, 0.2, 1.0),
                    Some(Border::new(params.background, 1.0 * params.scale.y))
                ).shape(Shape::Round(5.0))
            ))
        }

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &IcedRenderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &IcedRenderer,
        offset: iced::Vector
    ) -> Option<overlay::Element<'b, Message, iced::Theme, IcedRenderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            offset,
        )
    }
    
}

impl From<SkinnedButton> for IcedElement {
    fn from(value: SkinnedButton) -> Self {
        IcedElement::new(value)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SkinParams {
    pub text: Color,
    pub background: Color,
    pub scale: Vector2,
    pub offset: Vector2,
}
impl SkinParams {
    pub fn new(text: Color, background: Color, offset: Vector2, scale: Vector2) -> Self {
        Self {
            text,
            background, 
            scale,
            offset,
        }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.background = color;
        self
    }
    pub fn scale(mut self, scale: Vector2) -> Self {
        self.scale = scale;
        self
    }
    pub fn offset(mut self, offset: Vector2) -> Self {
        self.offset = offset;
        self
    }
}
impl Default for SkinParams {
    fn default() -> Self {
        Self::new(Color::BLACK, Color::new(0.2, 0.2, 0.2, 1.0), Vector2::ZERO, Vector2::ONE)
    }
}


/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_pressed: bool,
}
