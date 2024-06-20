use crate::prelude::*;
use iced::event::Status;
use iced::keyboard::Event;

// pub struct KeyEventsHandlerGroup<T:KeyMap> {
//     receiver: AsyncReceiver<KeyEvent<T>>,
//     handler: KeyEventsHandler<T>,
// }
// impl<T:KeyMap> KeyEventsHandlerGroup<T> {
//     pub fn new() -> Self {
//         let (handler, receiver) = KeyEventsHandler::new();
//         Self {
//             handler,
//             receiver,
//         }
//     }

//     pub fn handler(&self) -> KeyEventsHandler<T> {
//         self.handler.clone()
//     }

//     pub fn check_events(&mut self) -> Option<KeyEvent<T>> {
//         self.receiver.try_recv().ok()
//     }
// }


// pub struct KeyEventsHandler<T: KeyMap>(AsyncSender<KeyEvent<T>>);
// impl<T:KeyMap> KeyEventsHandler<T> {
//     pub fn new() -> (Self, AsyncReceiver<KeyEvent<T>>) {
//         let (sender, receiver) = async_channel(10);

//         (Self(sender), receiver)
//     }
// }
// // derive macro requires all generic arguments to be clone
// impl<T: KeyMap> Clone for KeyEventsHandler<T> {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl<T:KeyMap> iced::advanced::Widget<Message, iced::Theme, IcedRenderer> for KeyEventsHandler<T> {
//     fn width(&self) -> iced::Length { iced::Length::Fixed(0.0) }
//     fn height(&self) -> iced::Length { iced::Length::Fixed(0.0) }

//     fn on_event(
//         &mut self,
//         _state: &mut iced_core::widget::Tree,
//         event: iced::Event,
//         _layout: iced_core::Layout<'_>,
//         _cursor: iced_core::mouse::Cursor,
//         _renderer: &IcedRenderer,
//         _clipboard: &mut dyn iced_core::Clipboard,
//         _shell: &mut iced_core::Shell<'_, Message>,
//         _viewport: &iced::Rectangle,
//     ) -> Status {
//         let iced::Event::Keyboard(event) = event else { return Status::Ignored };
//         match event {
//             Event::KeyPressed { key_code, modifiers } => {
//                 let Some(event) = T::from_key(key_code, modifiers) else { return Status::Ignored };
//                 let _ = self.0.try_send(KeyEvent::Press(event));
//                 Status::Captured
//             }

//             Event::KeyReleased { key_code, modifiers } => {
//                 let Some(event) = T::from_key(key_code, modifiers) else { return Status::Ignored };
//                 let _ = self.0.try_send(KeyEvent::Release(event));
//                 Status::Captured
//             }

//             Event::CharacterReceived(char) if T::handle_chars() => {
//                 let _ = self.0.try_send(KeyEvent::Char(char));
//                 Status::Captured
//             }

//             _ => Status::Ignored
//         }
//     }

//     fn layout(
//         &self,
//         _renderer: &IcedRenderer,
//         _limits: &iced_core::layout::Limits,
//     ) -> iced_core::layout::Node {
//         iced_core::layout::Node::default()
//     }

//     fn draw(
//         &self,
//         _state: &iced_core::widget::Tree,
//         _renderer: &mut IcedRenderer,
//         _theme: &iced::Theme,
//         _style: &iced_core::renderer::Style,
//         _layout: iced_core::Layout<'_>,
//         _cursor: iced_core::mouse::Cursor,
//         _viewport: &iced::Rectangle,
//     ) {}
// }

// impl<T:KeyMap + 'static> From<KeyEventsHandler<T>> for IcedElement {
//     fn from(value: KeyEventsHandler<T>) -> Self {
//         Self::new(value)
//     }
// }

// pub trait KeyMap: Sized + Send + Sync {
//     fn handle_chars() -> bool { true }
//     fn from_key(key: iced::keyboard::KeyCode, mods: iced::keyboard::Modifiers) -> Option<Self>;
// }
// pub enum KeyEvent<T:KeyMap> {
//     Press(T),
//     Release(T),
//     Char(char),
// }

#[derive(Clone)]
pub struct KeyEvent {
    key: Key,
    mods: KeyModifiers,
    message: Message
}
impl KeyEvent {
    pub fn new(key: Key, mods: KeyModifiers, message: Message) -> Self {
        Self {
            key, 
            mods, 
            message
        }
    }
}

#[derive(Clone)]
pub struct KeyEventsHandler(Vec<KeyEvent>);
impl KeyEventsHandler {
    pub fn new(events: &Vec<KeyHandlerEvent>, owner: MessageOwner, values: &mut ValueCollection) -> Self {
        Self(
            events
            .iter()
            .filter_map(|a| 
                a.action
                .resolve(owner, values, None)
                .map(|message| KeyEvent::new(a.key, a.mods, message))
            )
            .collect()
        )
    }
}

impl iced::advanced::Widget<Message, iced::Theme, IcedRenderer> for KeyEventsHandler {
    fn size(&self) -> iced::Size<iced::Length> { iced::Size::new(iced::Length::Fixed(0.0), iced::Length::Fixed(0.0)) }

    fn on_event(
        &mut self,
        _state: &mut iced_core::widget::Tree,
        event: iced::Event,
        _layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _renderer: &IcedRenderer,
        _clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> Status {
        let iced::Event::Keyboard(event) = event else { return Status::Ignored };
        match event {
            // Event::KeyPressed { key_code, modifiers } => {
            //     let key = keyboard(key_code);
            //     let mods:KeyModifiers = modifiers.into();

            //     for i in self.0.iter() {
            //         if i.key != key || i.mods != mods { continue }

            //         shell.publish(i.message.clone());
            //         return Status::Captured;
            //     }

            //     // let Some(event) = T::from_key(key_code, modifiers) else { return Status::Ignored };
            //     // let _ = self.0.try_send(KeyEvent::Press(event));
            //     // Status::Captured
            //     Status::Ignored
            // }

            // Event::KeyReleased { key_code, modifiers } => {
            //     let Some(event) = T::from_key(key_code, modifiers) else { return Status::Ignored };
            //     let _ = self.0.try_send(KeyEvent::Release(event));
            //     Status::Captured
            // }

            // Event::CharacterReceived(char) if T::handle_chars() => {
            //     let _ = self.0.try_send(KeyEvent::Char(char));
            //     Status::Captured
            // }

            _ => Status::Ignored
        }
    }

    fn layout(
        &self,
        _state: &mut iced_core::widget::Tree,
        _renderer: &IcedRenderer,
        _limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        iced_core::layout::Node::default()
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
}

impl From<KeyEventsHandler> for IcedElement {
    fn from(value: KeyEventsHandler) -> Self {
        Self::new(value)
    }
}






// fn keyboard(key: iced::keyboard::KeyCode) -> Key {
//     match key {
//         iced::keyboard::KeyCode::Key1 => Key::Key1,
//         iced::keyboard::KeyCode::Key2 => Key::Key2,
//         iced::keyboard::KeyCode::Key3 => Key::Key3,
//         iced::keyboard::KeyCode::Key4 => Key::Key4,
//         iced::keyboard::KeyCode::Key5 => Key::Key5,
//         iced::keyboard::KeyCode::Key6 => Key::Key6,
//         iced::keyboard::KeyCode::Key7 => Key::Key7,
//         iced::keyboard::KeyCode::Key8 => Key::Key8,
//         iced::keyboard::KeyCode::Key9 => Key::Key9,
//         iced::keyboard::KeyCode::Key0 => Key::Key0,
//         iced::keyboard::KeyCode::A => Key::A,
//         iced::keyboard::KeyCode::B => Key::B,
//         iced::keyboard::KeyCode::C => Key::C,
//         iced::keyboard::KeyCode::D => Key::D,
//         iced::keyboard::KeyCode::E => Key::E,
//         iced::keyboard::KeyCode::F => Key::F,
//         iced::keyboard::KeyCode::G => Key::G,
//         iced::keyboard::KeyCode::H => Key::H,
//         iced::keyboard::KeyCode::I => Key::I,
//         iced::keyboard::KeyCode::J => Key::J,
//         iced::keyboard::KeyCode::K => Key::K,
//         iced::keyboard::KeyCode::L => Key::L,
//         iced::keyboard::KeyCode::M => Key::M,
//         iced::keyboard::KeyCode::N => Key::N,
//         iced::keyboard::KeyCode::O => Key::O,
//         iced::keyboard::KeyCode::P => Key::P,
//         iced::keyboard::KeyCode::Q => Key::Q,
//         iced::keyboard::KeyCode::R => Key::R,
//         iced::keyboard::KeyCode::S => Key::S,
//         iced::keyboard::KeyCode::T => Key::T,
//         iced::keyboard::KeyCode::U => Key::U,
//         iced::keyboard::KeyCode::V => Key::V,
//         iced::keyboard::KeyCode::W => Key::W,
//         iced::keyboard::KeyCode::X => Key::X,
//         iced::keyboard::KeyCode::Y => Key::Y,
//         iced::keyboard::KeyCode::Z => Key::Z,
//         iced::keyboard::KeyCode::Escape => Key::Escape,
//         iced::keyboard::KeyCode::F1 => Key::F1,
//         iced::keyboard::KeyCode::F2 => Key::F2,
//         iced::keyboard::KeyCode::F3 => Key::F3,
//         iced::keyboard::KeyCode::F4 => Key::F4,
//         iced::keyboard::KeyCode::F5 => Key::F5,
//         iced::keyboard::KeyCode::F6 => Key::F6,
//         iced::keyboard::KeyCode::F7 => Key::F7,
//         iced::keyboard::KeyCode::F8 => Key::F8,
//         iced::keyboard::KeyCode::F9 => Key::F9,
//         iced::keyboard::KeyCode::F10 => Key::F10,
//         iced::keyboard::KeyCode::F11 => Key::F11,
//         iced::keyboard::KeyCode::F12 => Key::F12,
//         iced::keyboard::KeyCode::F13 => Key::F13,
//         iced::keyboard::KeyCode::F14 => Key::F14,
//         iced::keyboard::KeyCode::F15 => Key::F15,
//         iced::keyboard::KeyCode::F16 => Key::F16,
//         iced::keyboard::KeyCode::F17 => Key::F17,
//         iced::keyboard::KeyCode::F18 => Key::F18,
//         iced::keyboard::KeyCode::F19 => Key::F19,
//         iced::keyboard::KeyCode::F20 => Key::F20,
//         iced::keyboard::KeyCode::F21 => Key::F21,
//         iced::keyboard::KeyCode::F22 => Key::F22,
//         iced::keyboard::KeyCode::F23 => Key::F23,
//         iced::keyboard::KeyCode::F24 => Key::F24,
//         iced::keyboard::KeyCode::Snapshot => Key::Snapshot,
//         iced::keyboard::KeyCode::Scroll => Key::Scroll,
//         iced::keyboard::KeyCode::Pause => Key::Pause,
//         iced::keyboard::KeyCode::Insert => Key::Insert,
//         iced::keyboard::KeyCode::Home => Key::Home,
//         iced::keyboard::KeyCode::Delete => Key::Delete,
//         iced::keyboard::KeyCode::End => Key::End,
//         iced::keyboard::KeyCode::PageDown => Key::PageDown,
//         iced::keyboard::KeyCode::PageUp => Key::PageUp,
//         iced::keyboard::KeyCode::Left => Key::Left,
//         iced::keyboard::KeyCode::Up => Key::Up,
//         iced::keyboard::KeyCode::Right => Key::Right,
//         iced::keyboard::KeyCode::Down => Key::Down,
//         iced::keyboard::KeyCode::Backspace => Key::Back,
//         iced::keyboard::KeyCode::Enter => Key::Return,
//         iced::keyboard::KeyCode::Space => Key::Space,
//         iced::keyboard::KeyCode::Compose => Key::Compose,
//         iced::keyboard::KeyCode::Caret => Key::Caret,
//         iced::keyboard::KeyCode::Numlock => Key::Numlock,
//         iced::keyboard::KeyCode::Numpad0 => Key::Numpad0,
//         iced::keyboard::KeyCode::Numpad1 => Key::Numpad1,
//         iced::keyboard::KeyCode::Numpad2 => Key::Numpad2,
//         iced::keyboard::KeyCode::Numpad3 => Key::Numpad3,
//         iced::keyboard::KeyCode::Numpad4 => Key::Numpad4,
//         iced::keyboard::KeyCode::Numpad5 => Key::Numpad5,
//         iced::keyboard::KeyCode::Numpad6 => Key::Numpad6,
//         iced::keyboard::KeyCode::Numpad7 => Key::Numpad7,
//         iced::keyboard::KeyCode::Numpad8 => Key::Numpad8,
//         iced::keyboard::KeyCode::Numpad9 => Key::Numpad9,
//         iced::keyboard::KeyCode::NumpadAdd => Key::NumpadAdd,
//         iced::keyboard::KeyCode::NumpadDivide => Key::NumpadDivide,
//         iced::keyboard::KeyCode::NumpadDecimal => Key::NumpadDecimal,
//         iced::keyboard::KeyCode::NumpadComma => Key::NumpadComma,
//         iced::keyboard::KeyCode::NumpadEnter => Key::NumpadEnter,
//         iced::keyboard::KeyCode::NumpadEquals => Key::NumpadEquals,
//         iced::keyboard::KeyCode::NumpadMultiply => Key::NumpadMultiply,
//         iced::keyboard::KeyCode::NumpadSubtract => Key::NumpadSubtract,
//         iced::keyboard::KeyCode::AbntC1 => Key::AbntC1,
//         iced::keyboard::KeyCode::AbntC2 => Key::AbntC2,
//         iced::keyboard::KeyCode::Apostrophe => Key::Apostrophe,
//         iced::keyboard::KeyCode::Apps => Key::Apps,
//         iced::keyboard::KeyCode::Asterisk => Key::Asterisk,
//         iced::keyboard::KeyCode::At => Key::At,
//         iced::keyboard::KeyCode::Ax => Key::Ax,
//         iced::keyboard::KeyCode::Backslash => Key::Backslash,
//         iced::keyboard::KeyCode::Calculator => Key::Calculator,
//         iced::keyboard::KeyCode::Capital => Key::Capital,
//         iced::keyboard::KeyCode::Colon => Key::Colon,
//         iced::keyboard::KeyCode::Comma => Key::Comma,
//         iced::keyboard::KeyCode::Convert => Key::Convert,
//         iced::keyboard::KeyCode::Equals => Key::Equals,
//         iced::keyboard::KeyCode::Grave => Key::Grave,
//         iced::keyboard::KeyCode::Kana => Key::Kana,
//         iced::keyboard::KeyCode::Kanji => Key::Kanji,
//         iced::keyboard::KeyCode::LAlt => Key::LAlt,
//         iced::keyboard::KeyCode::LBracket => Key::LBracket,
//         iced::keyboard::KeyCode::LControl => Key::LControl,
//         iced::keyboard::KeyCode::LShift => Key::LShift,
//         iced::keyboard::KeyCode::LWin => Key::LWin,
//         iced::keyboard::KeyCode::Mail => Key::Mail,
//         iced::keyboard::KeyCode::MediaSelect => Key::MediaSelect,
//         iced::keyboard::KeyCode::MediaStop => Key::MediaStop,
//         iced::keyboard::KeyCode::Minus => Key::Minus,
//         iced::keyboard::KeyCode::Mute => Key::Mute,
//         iced::keyboard::KeyCode::MyComputer => Key::MyComputer,
//         iced::keyboard::KeyCode::NavigateForward => Key::NavigateForward,  // also called "Next",
//         iced::keyboard::KeyCode::NavigateBackward => Key::NavigateBackward, // also called "Prior",
//         iced::keyboard::KeyCode::NextTrack => Key::NextTrack,
//         iced::keyboard::KeyCode::NoConvert => Key::NoConvert,
//         iced::keyboard::KeyCode::OEM102 => Key::OEM102,
//         iced::keyboard::KeyCode::Period => Key::Period,
//         iced::keyboard::KeyCode::PlayPause => Key::PlayPause,
//         iced::keyboard::KeyCode::Plus => Key::Plus,
//         iced::keyboard::KeyCode::Power => Key::Power,
//         iced::keyboard::KeyCode::PrevTrack => Key::PrevTrack,
//         iced::keyboard::KeyCode::RAlt => Key::RAlt,
//         iced::keyboard::KeyCode::RBracket => Key::RBracket,
//         iced::keyboard::KeyCode::RControl => Key::RControl,
//         iced::keyboard::KeyCode::RShift => Key::RShift,
//         iced::keyboard::KeyCode::RWin => Key::RWin,
//         iced::keyboard::KeyCode::Semicolon => Key::Semicolon,
//         iced::keyboard::KeyCode::Slash => Key::Slash,
//         iced::keyboard::KeyCode::Sleep => Key::Sleep,
//         iced::keyboard::KeyCode::Stop => Key::Stop,
//         iced::keyboard::KeyCode::Sysrq => Key::Sysrq,
//         iced::keyboard::KeyCode::Tab => Key::Tab,
//         iced::keyboard::KeyCode::Underline => Key::Underline,
//         iced::keyboard::KeyCode::Unlabeled => Key::Unlabeled,
//         iced::keyboard::KeyCode::VolumeDown => Key::VolumeDown,
//         iced::keyboard::KeyCode::VolumeUp => Key::VolumeUp,
//         iced::keyboard::KeyCode::Wake => Key::Wake,
//         iced::keyboard::KeyCode::WebBack => Key::WebBack,
//         iced::keyboard::KeyCode::WebFavorites => Key::WebFavorites,
//         iced::keyboard::KeyCode::WebForward => Key::WebForward,
//         iced::keyboard::KeyCode::WebHome => Key::WebHome,
//         iced::keyboard::KeyCode::WebRefresh => Key::WebRefresh,
//         iced::keyboard::KeyCode::WebSearch => Key::WebSearch,
//         iced::keyboard::KeyCode::WebStop => Key::WebStop,
//         iced::keyboard::KeyCode::Yen => Key::Yen,
//         iced::keyboard::KeyCode::Copy => Key::Copy,
//         iced::keyboard::KeyCode::Paste => Key::Paste,
//         iced::keyboard::KeyCode::Cut => Key::Cut,
//     }
// }
