use crate::prelude::*;
use iced::event::Status;
use iced::keyboard::Event;

pub struct KeyPressHandlerGroup<T:KeyMap> {
    receiver: AsyncReceiver<KeyEvent<T>>,
    handler: KeyPressHandler<T>,
}
impl<T:KeyMap> KeyPressHandlerGroup<T> {
    pub fn new() -> Self {
        let (handler, receiver) = KeyPressHandler::new();
        Self {
            handler,
            receiver,
        }
    }

    pub fn handler(&self) -> KeyPressHandler<T> {
        self.handler.clone()
    }

    pub fn check_events(&mut self) -> Option<KeyEvent<T>> {
        self.receiver.try_recv().ok()
    }
}


pub struct KeyPressHandler<T: KeyMap>(AsyncSender<KeyEvent<T>>);
impl<T:KeyMap> KeyPressHandler<T> {
    pub fn new() -> (Self, AsyncReceiver<KeyEvent<T>>) {
        let (sender, receiver) = async_channel(10);

        (Self(sender), receiver)
    }
}
// derive macro requires all generic arguments to be clone
impl<T: KeyMap> Clone for KeyPressHandler<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T:KeyMap> iced::advanced::Widget<Message, IcedRenderer> for KeyPressHandler<T> {
    fn width(&self) -> iced::Length { iced::Length::Fixed(0.0) }
    fn height(&self) -> iced::Length { iced::Length::Fixed(0.0) }

    fn on_event(
            &mut self,
            _state: &mut iced_runtime::core::widget::Tree,
            event: iced::Event,
            _layout: iced_runtime::core::Layout<'_>,
            _cursor: iced_runtime::core::mouse::Cursor,
            _renderer: &IcedRenderer,
            _clipboard: &mut dyn iced_runtime::core::Clipboard,
            _shell: &mut iced_runtime::core::Shell<'_, Message>,
            _viewport: &iced::Rectangle,
        ) -> Status {
        let iced::Event::Keyboard(event) = event else { return Status::Ignored };
        match event {
            Event::KeyPressed { key_code, modifiers } => {
                let Some(event) = T::from_key(key_code, modifiers) else { return Status::Ignored };
                let _ = self.0.try_send(KeyEvent::Press(event));
                Status::Captured
            }

            Event::KeyReleased { key_code, modifiers } => {
                let Some(event) = T::from_key(key_code, modifiers) else { return Status::Ignored };
                let _ = self.0.try_send(KeyEvent::Release(event));
                Status::Captured
            }

            Event::CharacterReceived(char) if T::handle_chars() => {
                let _ = self.0.try_send(KeyEvent::Char(char));
                Status::Captured
            }

            _ => Status::Ignored
        }
    }

    fn layout(
        &self,
        _renderer: &IcedRenderer,
        _limits: &iced_runtime::core::layout::Limits,
    ) -> iced_runtime::core::layout::Node {
        iced_runtime::core::layout::Node::default()
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
}

impl<T:KeyMap + 'static> From<KeyPressHandler<T>> for IcedElement {
    fn from(value: KeyPressHandler<T>) -> Self {
        Self::new(value)
    }
}

pub trait KeyMap: Sized + Send + Sync {
    fn handle_chars() -> bool { true }
    fn from_key(key: iced::keyboard::KeyCode, mods: iced::keyboard::Modifiers) -> Option<Self>;
}
pub enum KeyEvent<T:KeyMap> {
    Press(T),
    Release(T),
    Char(char),
}
