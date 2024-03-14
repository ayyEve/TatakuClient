use iced::Event;
use crate::prelude::*;
use tokio::sync::oneshot;
use iced::advanced::graphics::Primitive;
use iced_runtime::{ user_interface, UserInterface };

use iced::advanced::widget::Operation;

pub type IcedElement = iced::Element<'static, Message, IcedRenderer>;
pub type IcedOverlay<'a> = iced::overlay::Element<'a, Message, IcedRenderer>;
pub type IcedOperation = Box<dyn Operation<Message> + Send + Sync>;

pub struct UiManager {
    message_channel: (AsyncSender<Message>, AsyncReceiver<Message>),
    ui_sender: Sender<UiAction>,

    application: Option<UiApplication>,
    messages: Vec<Message>,
    current_menu: MenuType,

    queued_operations: Vec<IcedOperation>,
}
impl UiManager {
    pub fn new() -> Self {
        let (ui_sender, ui_receiver) = channel();

        // todo: store handle?
        tokio::task::spawn_blocking(move || { // std::thread::spawn(move || {
            Self::handle_actions(ui_receiver);
        });

        Self {
            // ui
            message_channel: async_channel(10),
            ui_sender,

            application: Some(UiApplication::new()),
            messages: Vec::new(),
            current_menu: MenuType::Internal("None"),

            queued_operations: Vec::new()
        }
    }
    pub fn set_menu(&mut self, menu: Box<dyn AsyncMenu>) {
        self.current_menu = MenuType::from_menu(&menu);
        self.application.as_mut().unwrap().menu = menu;
    }

    pub fn get_menu(&self) -> MenuType {
        self.current_menu.clone()
    }
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    } 


    pub fn add_operation(&mut self, operation: IcedOperation) {
        self.queued_operations.push(operation)
    }

    fn handle_actions(ui_receiver: Receiver<UiAction>) {
        let mut renderer = IcedRenderer::new(IcedBackend::new());
        let mut window_size = WindowSizeHelper::new();

        // do we rebuild the ui next frame? (required if the ui was updated, adding new items to the view)
        let mut rebuild_next = false;
        let mut needs_render = true;
        let mut last_menu = String::new();
        let mut last_draw = TransformGroup::new(Vector2::ZERO);
        let mut mouse_pos = iced::Point::ORIGIN;

        let mut ui: UserInterface<Message, IcedRenderer> = user_interface::UserInterface::build(
            iced::widget::Column::new().into_element(),
            iced::Size::new(window_size.x, window_size.y),
            iced_runtime::user_interface::Cache::default(), // std::mem::take(&mut cache),
            &mut renderer,
        );

        while let Ok(ui_action) = ui_receiver.recv() {
            match ui_action {
                UiAction::Update { 
                    application, 
                    callback, 
                    mut messages, 
                    events, 
                    operations, 
                    mut values 
                } => {
                    // rebuild ui with the new application
                    if rebuild_next || application.menu.get_name() != last_menu {
                        last_menu = application.menu.get_name().to_owned();
                        rebuild_next = false;
                        needs_render = true;

                        ui = user_interface::UserInterface::build(
                            application.view(&mut values),
                            iced::Size::new(window_size.x, window_size.y),
                            ui.into_cache(), //std::mem::take(&mut cache),
                            &mut renderer,
                        );
                    }

                    // update bounds
                    if window_size.update() {
                        ui = ui.relayout(iced::Size::new(window_size.x, window_size.y), &mut renderer);
                    }

                    // perform operations
                    for mut operation in operations {
                        ui.operate(
                            &renderer,
                            &mut *operation
                        )
                    }

                    mouse_pos = iced::Point::new(events.mouse_pos.x, events.mouse_pos.y);
                    let (_s, e) = ui.update(
                        &events.events,
                        iced::mouse::Cursor::Available(mouse_pos),
                        &mut renderer,
                        &mut iced_runtime::core::clipboard::Null,
                        &mut messages
                    );

                    if events.force_refresh || !e.is_empty() {
                        rebuild_next = true;
                    }

                    let _ = callback.send(UiUpdateData {
                        application,
                        messages,
                        values
                    });
                }
                UiAction::Draw { application, callback } => {
                    if needs_render || true {
                        needs_render = false;

                        ui.draw(&mut renderer, &iced::Theme::Dark, &Default::default(), iced::mouse::Cursor::Available(mouse_pos));

                        let mut group = TransformGroup::new(Vector2::ZERO);
                        renderer.with_primitives(|_b, p| p.iter().for_each(|p|group.push_arced(into_renderable(p))));
                        last_draw = group;
                        last_draw.raw_draw = true;
                    }

                    let _ = callback.send(UiDrawData {
                        application,
                        transform_group: last_draw.clone()
                    });
                }
            }
        }
    }

    pub async fn update<'a>(&mut self, state: CurrentInputState<'a>, values: ShuntingYardValues) -> (Vec<MenuAction>, ShuntingYardValues) {
        while let Ok(e) = self.message_channel.1.try_recv() {
            // info!("adding message: {e:?}");
            self.messages.push(e);
        }

        let (sender, callback) = oneshot::channel();

        let Ok(_) = self.ui_sender.send(UiAction::Update {
            application: self.application.take().unwrap(),
            callback: sender,
            messages: self.messages.take(),
            events: state.into_events(),
            operations: self.queued_operations.take(),
            values,
            // we should probably return an error instead
        }) else { return (Vec::new(), ShuntingYardValues::default()); };

        // we should probably return an error instead
        let Ok(UiUpdateData { application, messages, mut values }) = callback.await else { return (Vec::new(), ShuntingYardValues::default()); };

        std::mem::swap(&mut self.application, &mut Some(application));

        let app = self.application();
        for m in messages {
            app.handle_message(m, &mut values).await;
        }

        let mut list = app.update(&mut values).await;
        list.extend(app.dialog_manager.update(&mut values).await);
        (list, values)
    }

    pub async fn draw(&mut self, list: &mut RenderableCollection) {
        let (sender, callback) = oneshot::channel();

        let Ok(_) = self.ui_sender.send(UiAction::Draw {
            application: self.application.take().unwrap(),
            callback: sender
        }) else { return; };

        let Ok(UiDrawData { application, transform_group }) = callback.await else { return; };

        std::mem::swap(&mut self.application, &mut Some(application));

        list.push(transform_group);
    }

    pub fn application(&mut self) -> &mut UiApplication {
        self.application.as_mut().unwrap()
    }
}

fn into_renderable(p: &Primitive<Arc<dyn TatakuRenderable>>) -> Arc<dyn TatakuRenderable> {
    match p {
        iced::advanced::graphics::Primitive::Text {
            content,
            bounds,
            color,
            size: font_size,
            line_height,
            font,
            horizontal_alignment,
            vertical_alignment,
            shaping: _
        } => {
            let height = line_height.to_absolute(iced::Pixels(*font_size)).0;
            
            let mut text = Text::new(
                Vector2::new(bounds.x, bounds.y),
                *font_size,
                content,
                Color::new(color.r, color.g, color.b, color.a),
                Font::from_iced(font)
            );

            match vertical_alignment {
                iced::alignment::Vertical::Bottom => text.pos.y -= height,
                iced::alignment::Vertical::Center => text.pos.y -= height / 2.0,
                iced::alignment::Vertical::Top => {}
            }
            match horizontal_alignment {
                iced::alignment::Horizontal::Left => {}
                iced::alignment::Horizontal::Center => text.pos.x += bounds.x - text.measure_text().x / 2.0,
                iced::alignment::Horizontal::Right => text.pos.x += bounds.x - text.measure_text().x,
            }
            
            Arc::new(text)
        }
        iced::advanced::graphics::Primitive::Quad { bounds, background, border_radius, border_width, border_color } => {
            Arc::new(Rectangle::new(
                Vector2::new(bounds.x, bounds.y),
                Vector2::new(bounds.width, bounds.height),
                match background {
                    iced::Background::Color(color) => color.into(),
                    _ => Color::TRANSPARENT_WHITE,
                },
                Some(Border::new(border_color.into(), *border_width))
            ).shape(Shape::RoundSep(*border_radius)))
        }
        iced::advanced::graphics::Primitive::Group { primitives } => {
            let mut group = TransformGroup::new(Vector2::ZERO);
            group.items.reserve(primitives.len());
            for p in primitives {
                group.push_arced(into_renderable(p))
            }

            Arc::new(group)
        }
        iced::advanced::graphics::Primitive::Clip { bounds, content } => {
            let mut group = TransformGroup::new(Vector2::ZERO);
            group.set_scissor(Some([bounds.x, bounds.y, bounds.width, bounds.height]));
            group.push_arced(into_renderable(content));
            Arc::new(group)
        }
        iced::advanced::graphics::Primitive::Translate { translation, content } => {
            let mut group = TransformGroup::new(Vector2::new(translation.x, translation.y));
            group.push_arced(into_renderable(content));
            Arc::new(group)
        }

        iced::advanced::graphics::Primitive::Cache { content } => {
            into_renderable(content)
        }
        // iced::advanced::graphics::Primitive::Image { handle, bounds } => {}
        iced::advanced::graphics::Primitive::Custom(i) => {
            i.clone()
        }
        
        _ => {
            Arc::new(TransformGroup::new(Vector2::ZERO))
        }
    }
}


enum UiAction {
    Update {
        application: UiApplication,
        callback: oneshot::Sender<UiUpdateData>,
        messages: Vec<Message>,
        events: SendEvents,
        operations: Vec<IcedOperation>,

        values: ShuntingYardValues
    },
    Draw {
        application: UiApplication,
        callback: oneshot::Sender<UiDrawData>,
    }
}

struct UiUpdateData {
    application: UiApplication,
    messages: Vec<Message>,
    values: ShuntingYardValues,
}

struct UiDrawData {
    application: UiApplication,
    transform_group: TransformGroup,
}

pub struct CurrentInputState<'a> {
    pub mouse_pos: Vector2,
    pub mouse_moved: bool,
    pub scroll_delta: f32,

    pub mouse_down: &'a Vec<MouseButton>,
    pub mouse_up: &'a Vec<MouseButton>,

    pub keys_down: &'a Vec<Key>,
    pub keys_up: &'a Vec<Key>,
    pub text: &'a String,

    pub mods: KeyModifiers,
}
impl<'a> CurrentInputState<'a> {
    fn into_events(self) -> SendEvents {
        use iced::mouse::Event as MouseEvent;
        use iced::keyboard::Event as KeyboardEvent;

        let mut force_refresh = false;
        force_refresh |= !self.mouse_down.is_empty();
        force_refresh |= !self.mouse_up.is_empty();
        force_refresh |= !self.keys_down.is_empty();
        force_refresh |= !self.keys_up.is_empty();

        let mut events = Vec::new();
        if self.mouse_moved {
            events.push(Event::Mouse(MouseEvent::CursorMoved {
                position: iced::Point::new(self.mouse_pos.x, self.mouse_pos.y)
            }));
        }
        if self.scroll_delta != 0.0 {
            events.push(Event::Mouse(MouseEvent::WheelScrolled {
                delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: self.scroll_delta }
            }));
        }


        for i in self.mouse_down {
            events.push(Event::Mouse(MouseEvent::ButtonPressed(mouse_button(*i))));
        }
        for i in self.mouse_up {
            events.push(Event::Mouse(MouseEvent::ButtonReleased(mouse_button(*i))));
        }

        let modifiers = self.mods.into();
        for key in self.keys_down {
            events.push(Event::Keyboard(KeyboardEvent::KeyPressed { key_code: keyboard(*key), modifiers }));
        }
        for key in self.keys_up {
            events.push(Event::Keyboard(KeyboardEvent::KeyReleased { key_code: keyboard(*key), modifiers }));
        }

        for char in self.text.chars() {
            events.push(Event::Keyboard(KeyboardEvent::CharacterReceived(char)));
        }
        // events.push(Event::Keyboard(KeyboardEvent::ModifiersChanged(())))

        SendEvents {
            mouse_pos: self.mouse_pos,
            events,
            force_refresh,
        }
    }
}

struct SendEvents {
    mouse_pos: Vector2,
    events: Vec<Event>,
    force_refresh: bool,
}




/// Replaces the regular [`Into`] trait for types that can be converted
/// to an element. This is necessary, because the `Into` trait does not
/// assume the renderer type needs to match, after all, you could write an
/// `Into` trait implementation to convert between those types. This trait,
/// unlike `Into`, will propagate and infer the correct renderer type.
pub trait IntoElement where Self: 'static {
    fn into_element(self) -> IcedElement;
}

impl<T> IntoElement for T where
    IcedElement: From<T>,
    T: 'static
{
    fn into_element(self) -> IcedElement {
        IcedElement::from(self)
    }
}

fn mouse_button(mb: MouseButton) -> iced::mouse::Button {
    match mb {
        MouseButton::Left => iced::mouse::Button::Left,
        MouseButton::Right => iced::mouse::Button::Right,
        MouseButton::Middle => iced::mouse::Button::Middle,
        MouseButton::Other(i) => iced::mouse::Button::Other(i)
    }
}
fn keyboard(key: Key) -> iced::keyboard::KeyCode {
    match key {
        Key::Key1 => iced::keyboard::KeyCode::Key1,
        Key::Key2 => iced::keyboard::KeyCode::Key2,
        Key::Key3 => iced::keyboard::KeyCode::Key3,
        Key::Key4 => iced::keyboard::KeyCode::Key4,
        Key::Key5 => iced::keyboard::KeyCode::Key5,
        Key::Key6 => iced::keyboard::KeyCode::Key6,
        Key::Key7 => iced::keyboard::KeyCode::Key7,
        Key::Key8 => iced::keyboard::KeyCode::Key8,
        Key::Key9 => iced::keyboard::KeyCode::Key9,
        Key::Key0 => iced::keyboard::KeyCode::Key0,
        Key::A => iced::keyboard::KeyCode::A,
        Key::B => iced::keyboard::KeyCode::B,
        Key::C => iced::keyboard::KeyCode::C,
        Key::D => iced::keyboard::KeyCode::D,
        Key::E => iced::keyboard::KeyCode::E,
        Key::F => iced::keyboard::KeyCode::F,
        Key::G => iced::keyboard::KeyCode::G,
        Key::H => iced::keyboard::KeyCode::H,
        Key::I => iced::keyboard::KeyCode::I,
        Key::J => iced::keyboard::KeyCode::J,
        Key::K => iced::keyboard::KeyCode::K,
        Key::L => iced::keyboard::KeyCode::L,
        Key::M => iced::keyboard::KeyCode::M,
        Key::N => iced::keyboard::KeyCode::N,
        Key::O => iced::keyboard::KeyCode::O,
        Key::P => iced::keyboard::KeyCode::P,
        Key::Q => iced::keyboard::KeyCode::Q,
        Key::R => iced::keyboard::KeyCode::R,
        Key::S => iced::keyboard::KeyCode::S,
        Key::T => iced::keyboard::KeyCode::T,
        Key::U => iced::keyboard::KeyCode::U,
        Key::V => iced::keyboard::KeyCode::V,
        Key::W => iced::keyboard::KeyCode::W,
        Key::X => iced::keyboard::KeyCode::X,
        Key::Y => iced::keyboard::KeyCode::Y,
        Key::Z => iced::keyboard::KeyCode::Z,
        Key::Escape => iced::keyboard::KeyCode::Escape,
        Key::F1 => iced::keyboard::KeyCode::F1,
        Key::F2 => iced::keyboard::KeyCode::F2,
        Key::F3 => iced::keyboard::KeyCode::F3,
        Key::F4 => iced::keyboard::KeyCode::F4,
        Key::F5 => iced::keyboard::KeyCode::F5,
        Key::F6 => iced::keyboard::KeyCode::F6,
        Key::F7 => iced::keyboard::KeyCode::F7,
        Key::F8 => iced::keyboard::KeyCode::F8,
        Key::F9 => iced::keyboard::KeyCode::F9,
        Key::F10 => iced::keyboard::KeyCode::F10,
        Key::F11 => iced::keyboard::KeyCode::F11,
        Key::F12 => iced::keyboard::KeyCode::F12,
        Key::F13 => iced::keyboard::KeyCode::F13,
        Key::F14 => iced::keyboard::KeyCode::F14,
        Key::F15 => iced::keyboard::KeyCode::F15,
        Key::F16 => iced::keyboard::KeyCode::F16,
        Key::F17 => iced::keyboard::KeyCode::F17,
        Key::F18 => iced::keyboard::KeyCode::F18,
        Key::F19 => iced::keyboard::KeyCode::F19,
        Key::F20 => iced::keyboard::KeyCode::F20,
        Key::F21 => iced::keyboard::KeyCode::F21,
        Key::F22 => iced::keyboard::KeyCode::F22,
        Key::F23 => iced::keyboard::KeyCode::F23,
        Key::F24 => iced::keyboard::KeyCode::F24,
        Key::Snapshot => iced::keyboard::KeyCode::Snapshot,
        Key::Scroll => iced::keyboard::KeyCode::Scroll,
        Key::Pause => iced::keyboard::KeyCode::Pause,
        Key::Insert => iced::keyboard::KeyCode::Insert,
        Key::Home => iced::keyboard::KeyCode::Home,
        Key::Delete => iced::keyboard::KeyCode::Delete,
        Key::End => iced::keyboard::KeyCode::End,
        Key::PageDown => iced::keyboard::KeyCode::PageDown,
        Key::PageUp => iced::keyboard::KeyCode::PageUp,
        Key::Left => iced::keyboard::KeyCode::Left,
        Key::Up => iced::keyboard::KeyCode::Up,
        Key::Right => iced::keyboard::KeyCode::Right,
        Key::Down => iced::keyboard::KeyCode::Down,
        Key::Back => iced::keyboard::KeyCode::Backspace,
        Key::Return => iced::keyboard::KeyCode::Enter,
        Key::Space => iced::keyboard::KeyCode::Space,
        Key::Compose => iced::keyboard::KeyCode::Compose,
        Key::Caret => iced::keyboard::KeyCode::Caret,
        Key::Numlock => iced::keyboard::KeyCode::Numlock,
        Key::Numpad0 => iced::keyboard::KeyCode::Numpad0,
        Key::Numpad1 => iced::keyboard::KeyCode::Numpad1,
        Key::Numpad2 => iced::keyboard::KeyCode::Numpad2,
        Key::Numpad3 => iced::keyboard::KeyCode::Numpad3,
        Key::Numpad4 => iced::keyboard::KeyCode::Numpad4,
        Key::Numpad5 => iced::keyboard::KeyCode::Numpad5,
        Key::Numpad6 => iced::keyboard::KeyCode::Numpad6,
        Key::Numpad7 => iced::keyboard::KeyCode::Numpad7,
        Key::Numpad8 => iced::keyboard::KeyCode::Numpad8,
        Key::Numpad9 => iced::keyboard::KeyCode::Numpad9,
        Key::NumpadAdd => iced::keyboard::KeyCode::NumpadAdd,
        Key::NumpadDivide => iced::keyboard::KeyCode::NumpadDivide,
        Key::NumpadDecimal => iced::keyboard::KeyCode::NumpadDecimal,
        Key::NumpadComma => iced::keyboard::KeyCode::NumpadComma,
        Key::NumpadEnter => iced::keyboard::KeyCode::NumpadEnter,
        Key::NumpadEquals => iced::keyboard::KeyCode::NumpadEquals,
        Key::NumpadMultiply => iced::keyboard::KeyCode::NumpadMultiply,
        Key::NumpadSubtract => iced::keyboard::KeyCode::NumpadSubtract,
        Key::AbntC1 => iced::keyboard::KeyCode::AbntC1,
        Key::AbntC2 => iced::keyboard::KeyCode::AbntC2,
        Key::Apostrophe => iced::keyboard::KeyCode::Apostrophe,
        Key::Apps => iced::keyboard::KeyCode::Apps,
        Key::Asterisk => iced::keyboard::KeyCode::Asterisk,
        Key::At => iced::keyboard::KeyCode::At,
        Key::Ax => iced::keyboard::KeyCode::Ax,
        Key::Backslash => iced::keyboard::KeyCode::Backslash,
        Key::Calculator => iced::keyboard::KeyCode::Calculator,
        Key::Capital => iced::keyboard::KeyCode::Capital,
        Key::Colon => iced::keyboard::KeyCode::Colon,
        Key::Comma => iced::keyboard::KeyCode::Comma,
        Key::Convert => iced::keyboard::KeyCode::Convert,
        Key::Equals => iced::keyboard::KeyCode::Equals,
        Key::Grave => iced::keyboard::KeyCode::Grave,
        Key::Kana => iced::keyboard::KeyCode::Kana,
        Key::Kanji => iced::keyboard::KeyCode::Kanji,
        Key::LAlt => iced::keyboard::KeyCode::LAlt,
        Key::LBracket => iced::keyboard::KeyCode::LBracket,
        Key::LControl => iced::keyboard::KeyCode::LControl,
        Key::LShift => iced::keyboard::KeyCode::LShift,
        Key::LWin => iced::keyboard::KeyCode::LWin,
        Key::Mail => iced::keyboard::KeyCode::Mail,
        Key::MediaSelect => iced::keyboard::KeyCode::MediaSelect,
        Key::MediaStop => iced::keyboard::KeyCode::MediaStop,
        Key::Minus => iced::keyboard::KeyCode::Minus,
        Key::Mute => iced::keyboard::KeyCode::Mute,
        Key::MyComputer => iced::keyboard::KeyCode::MyComputer,
        Key::NavigateForward => iced::keyboard::KeyCode::NavigateForward,  // also called "Next"
        Key::NavigateBackward => iced::keyboard::KeyCode::NavigateBackward, // also called "Prior"
        Key::NextTrack => iced::keyboard::KeyCode::NextTrack,
        Key::NoConvert => iced::keyboard::KeyCode::NoConvert,
        Key::OEM102 => iced::keyboard::KeyCode::OEM102,
        Key::Period => iced::keyboard::KeyCode::Period,
        Key::PlayPause => iced::keyboard::KeyCode::PlayPause,
        Key::Plus => iced::keyboard::KeyCode::Plus,
        Key::Power => iced::keyboard::KeyCode::Power,
        Key::PrevTrack => iced::keyboard::KeyCode::PrevTrack,
        Key::RAlt => iced::keyboard::KeyCode::RAlt,
        Key::RBracket => iced::keyboard::KeyCode::RBracket,
        Key::RControl => iced::keyboard::KeyCode::RControl,
        Key::RShift => iced::keyboard::KeyCode::RShift,
        Key::RWin => iced::keyboard::KeyCode::RWin,
        Key::Semicolon => iced::keyboard::KeyCode::Semicolon,
        Key::Slash => iced::keyboard::KeyCode::Slash,
        Key::Sleep => iced::keyboard::KeyCode::Sleep,
        Key::Stop => iced::keyboard::KeyCode::Stop,
        Key::Sysrq => iced::keyboard::KeyCode::Sysrq,
        Key::Tab => iced::keyboard::KeyCode::Tab,
        Key::Underline => iced::keyboard::KeyCode::Underline,
        Key::Unlabeled => iced::keyboard::KeyCode::Unlabeled,
        Key::VolumeDown => iced::keyboard::KeyCode::VolumeDown,
        Key::VolumeUp => iced::keyboard::KeyCode::VolumeUp,
        Key::Wake => iced::keyboard::KeyCode::Wake,
        Key::WebBack => iced::keyboard::KeyCode::WebBack,
        Key::WebFavorites => iced::keyboard::KeyCode::WebFavorites,
        Key::WebForward => iced::keyboard::KeyCode::WebForward,
        Key::WebHome => iced::keyboard::KeyCode::WebHome,
        Key::WebRefresh => iced::keyboard::KeyCode::WebRefresh,
        Key::WebSearch => iced::keyboard::KeyCode::WebSearch,
        Key::WebStop => iced::keyboard::KeyCode::WebStop,
        Key::Yen => iced::keyboard::KeyCode::Yen,
        Key::Copy => iced::keyboard::KeyCode::Copy,
        Key::Paste => iced::keyboard::KeyCode::Paste,
        Key::Cut => iced::keyboard::KeyCode::Cut,
        // _ => unimplemented!()
    }
}



mod macros {
    // idk why this says its unused, if i remove it everything cries
    #[allow(unused)]
    use crate::prelude::*;
    
    #[macro_export]
    macro_rules! row {
        ($($i:expr),*;$($t:ident = $v:expr),*) => {
            iced::widget::Row::with_children(vec![
                $(
                    $i.into_element(),
                )*
            ]) 
            $(
                .$t($v)
            )*

            .into_element()
        };

        ($vec:expr, $($t:ident = $v:expr),*) => {
            iced::widget::Row::with_children($vec)
            $(
                .$t($v)
            )*

            .into_element()
        }
    }

    #[macro_export]
    macro_rules! col {
        ($($i:expr),*;$($t:ident = $v:expr),*) => {
            iced::widget::Column::with_children(vec![
                $(
                    $i.into_element(),
                )*
            ]) 
            $(
                .$t($v)
            )*

            .into_element()
        };

        ($vec:expr, $($t:ident = $v:expr),*) => {
            iced::widget::Column::with_children($vec)
            $(
                .$t($v)
            )*

            .into_element()
        }
    }

    #[cfg(test)]
    #[allow(unused)]
    fn test() {
        use crate::prelude::iced_elements::*;

        let row = row!(
            Space::new(Fill, Fill),
            Space::new(Fill, Fill);
            width = Fill,
            height = Fill
        );

        let col = col!(
            Space::new(Fill, Fill),
            Space::new(Fill, Fill);
            width = Fill,
            height = Fill
        );
    }

}
