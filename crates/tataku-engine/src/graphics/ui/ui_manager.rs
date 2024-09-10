
use crate::prelude::*;
use tokio::sync::oneshot;
use iced::Event;
// use iced::advanced::graphics::Primitive;
use iced_runtime::{ user_interface, UserInterface };

use iced::advanced::widget::Operation; 

pub type IcedElement = iced::Element<'static, Message, iced::Theme, IcedRenderer>;
pub type IcedOverlay<'a> = iced::overlay::Element<'a, Message, iced::Theme, IcedRenderer>;
pub type IcedOperation = Box<dyn Operation<Message> + Send + Sync>;

pub struct UiManager<T:Reflect> {
    pub force_refresh: bool,
    message_channel: (AsyncSender<Message>, AsyncReceiver<Message>),
    ui_sender: Sender<UiAction<T>>,

    application: Option<UiApplication>,
    messages: Vec<Message>,
    current_menu: MenuType,

    queued_operations: Vec<IcedOperation>,
}
impl<T:Reflect> UiManager<T> {
    pub fn new() -> Self {
        let (ui_sender, ui_receiver) = channel();

        // todo: store handle?
        tokio::task::spawn_blocking(move || { // std::thread::spawn(move || {
            Self::handle_actions(ui_receiver);
        });

        Self {
            // ui
            force_refresh: false,
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
        self.messages.retain(|m| !m.owner.is_menu())
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

    fn handle_actions(ui_receiver: Receiver<UiAction<T>>) {
        let mut renderer = IcedRenderer::new();
        let mut window_size = WindowSizeHelper::new();

        // do we rebuild the ui next frame? (required if the ui was updated, adding new items to the view)
        let mut rebuild_next = false;
        let mut needs_render = true;
        let mut last_menu = String::new();
        let mut last_draw = TransformGroup::new(Vector2::ZERO);
        let mut mouse_pos = iced::Point::ORIGIN;


        let mut ui: UserInterface<Message, iced::Theme, IcedRenderer> = user_interface::UserInterface::build(
            iced::widget::Column::new().into_element(),
            iced::Size::new(window_size.x, window_size.y),
            iced_runtime::user_interface::Cache::default(),
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
                    mut values ,
                    force_refresh,
                } => {
                    // rebuild ui with the new application
                    if force_refresh || rebuild_next || application.menu.get_name() != last_menu {
                        last_menu = application.menu.get_name().to_owned();
                        rebuild_next = false;
                        needs_render = true;

                        ui = user_interface::UserInterface::build(
                            application.view(&mut values),
                            iced::Size::new(window_size.x, window_size.y),
                            ui.into_cache(), 
                            &mut renderer,
                        );
                    }

                    // update bounds
                    if window_size.update() {
                        ui = ui.relayout(iced::Size::new(window_size.x, window_size.y), &mut renderer);
                    }

                    // perform operations
                    for mut operation in operations {
                        ui.operate(&renderer, &mut *operation)
                    }

                    mouse_pos = iced::Point::new(events.mouse_pos.x, events.mouse_pos.y);
                    let (_s, e) = ui.update(
                        &events.window_events,
                        iced::mouse::Cursor::Available(mouse_pos),
                        &mut renderer,
                        &mut iced_core::clipboard::Null,
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

                        ui.draw(
                            &mut renderer, 
                            &iced::Theme::Dark, 
                            &Default::default(), 
                            iced::mouse::Cursor::Available(mouse_pos)
                        );

                        // renderer.with_primitives(|_b, p| p.iter().for_each(|p| group.push_arced(into_renderable(p))));
                        last_draw = renderer.finish();
                        // last_draw.raw_draw = true;
                    }

                    let _ = callback.send(UiDrawData {
                        application,
                        transform_group: last_draw.clone()
                    });
                }
            }
        }
    }

    pub async fn update<'a>(
        &mut self, 
        state: CurrentInputState<'a>, 
        tataku_events: Vec<(TatakuEventType, Option<TatakuValue>)>,
        values: T,
    ) -> (Vec<TatakuAction>, T) {
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
            force_refresh: self.force_refresh,
            // we should probably return an error instead
        }) else { panic!("fucked up"); };
        self.force_refresh = false;

        // we should probably return an error instead
        let Ok(UiUpdateData { application, messages, mut values }) = callback.await else { 
            panic!("fucked up"); 
        };

        std::mem::swap(&mut self.application, &mut Some(application));

        let app = self.application();
        for m in messages {
            app.handle_message(m, &mut values).await;
        }

        for (event, param) in tataku_events {
            // debug!("handling event {event:?}");
            app.handle_event(event, param, &mut values).await;
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

// fn into_renderable(p: &Primitive<Arc<dyn TatakuRenderable>>) -> Arc<dyn TatakuRenderable> {
//     match p {
//         iced::advanced::graphics::Primitive::Text {
//             content,
//             bounds,
//             color,
//             size: font_size,
//             line_height,
//             font,
//             horizontal_alignment,
//             vertical_alignment,
//             shaping: _,
//         } => {
//             let height = line_height.to_absolute(iced::Pixels(*font_size)).0;
            
//             let mut text = Text::new(
//                 Vector2::new(bounds.x, bounds.y),
//                 *font_size,
//                 content,
//                 Color::new(color.r, color.g, color.b, color.a),
//                 crate::prelude::Font::from_iced(font)
//             );

//             match vertical_alignment {
//                 iced::alignment::Vertical::Bottom => text.pos.y -= height,
//                 iced::alignment::Vertical::Center => text.pos.y -= height / 2.0,
//                 iced::alignment::Vertical::Top => {}
//             }
//             match horizontal_alignment {
//                 iced::alignment::Horizontal::Left => {}
//                 iced::alignment::Horizontal::Center => text.pos.x += bounds.x - text.measure_text().x / 2.0,
//                 iced::alignment::Horizontal::Right => text.pos.x += bounds.x - text.measure_text().x,
//             }
            
//             Arc::new(text)
//         }
//         iced::advanced::graphics::Primitive::Quad { 
//             bounds, 
//             background, 
//             border_radius, 
//             border_width, 
//             border_color ,
//         } => {
//             Arc::new(Rectangle::new(
//                 Vector2::new(bounds.x, bounds.y),
//                 Vector2::new(bounds.width, bounds.height),
//                 match background {
//                     iced::Background::Color(color) => color.into(),
//                     _ => Color::TRANSPARENT_WHITE,
//                 },
//                 Some(Border::new(border_color.into(), *border_width))
//             ).shape(Shape::RoundSep(*border_radius)))
//         }
//         iced::advanced::graphics::Primitive::Group { primitives } => {
//             let mut group = TransformGroup::new(Vector2::ZERO);
//             group.items.reserve(primitives.len());
//             for p in primitives {
//                 group.push_arced(into_renderable(p))
//             }

//             Arc::new(group)
//         }
//         iced::advanced::graphics::Primitive::Clip { bounds, content } => {
//             let mut group = ScissorGroup::new();
//             // group.set_scissor(Some([bounds.x, bounds.y, bounds.width, bounds.height]));
//             group.push_arced(into_renderable(content));
//             Arc::new(group)
//         }
//         iced::advanced::graphics::Primitive::Translate { translation, content } => {
//             let mut group = TransformGroup::new(Vector2::new(translation.x, translation.y));
//             group.push_arced(into_renderable(content));
//             Arc::new(group)
//         }

//         iced::advanced::graphics::Primitive::Cache { content } => {
//             into_renderable(content)
//         }
//         // iced::advanced::graphics::Primitive::Image { handle, bounds } => {}
//         iced::advanced::graphics::Primitive::Custom(i) => {
//             i.clone()
//         }
        
//         _ => {
//             Arc::new(TransformGroup::new(Vector2::ZERO))
//         }
//     }
// }


enum UiAction<T:Reflect> {
    Update {
        application: UiApplication,
        callback: oneshot::Sender<UiUpdateData<T>>,
        messages: Vec<Message>,
        events: SendEvents,
        operations: Vec<IcedOperation>,

        values: T,
        force_refresh: bool,
    },
    Draw {
        application: UiApplication,
        callback: oneshot::Sender<UiDrawData>,
    }
}

struct UiUpdateData<T: Reflect> {
    application: UiApplication,
    messages: Vec<Message>,
    values: T,
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

    pub keys_down: &'a KeyCollection,
    pub keys_up: &'a KeyCollection,

    pub mods: KeyModifiers,
}
impl<'a> CurrentInputState<'a> {
    fn into_events(self) -> SendEvents {
        use iced::mouse::Event as MouseEvent;
        use iced::keyboard::Event as KeyboardEvent;

        let mut force_refresh = false;
        force_refresh |= !self.mouse_down.is_empty();
        force_refresh |= !self.mouse_up.is_empty();
        force_refresh |= !self.keys_down.0.is_empty();
        force_refresh |= !self.keys_up.0.is_empty();

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


        for i in self.mouse_down.iter().filter_map(mouse_button) {
            events.push(Event::Mouse(MouseEvent::ButtonPressed(i)));
        }
        for i in self.mouse_up.iter().filter_map(mouse_button) {
            events.push(Event::Mouse(MouseEvent::ButtonReleased(i)));
        }

        let modifiers = self.mods.into();
        for key in &self.keys_down.0 {
            events.push(Event::Keyboard(KeyboardEvent::KeyPressed { 
                key: conv_key(key.logical.clone()), 
                location: conv_location(key.location), 
                text: key.text.clone(),
                modifiers,
            }));
        }
        for key in &self.keys_up.0 {
            events.push(Event::Keyboard(KeyboardEvent::KeyReleased { 
                key: conv_key(key.logical.clone()), 
                location: conv_location(key.location), 
                modifiers,
            }));
        }

        SendEvents {
            mouse_pos: self.mouse_pos,
            window_events: events,
            force_refresh,
        }
    }
}

struct SendEvents {
    mouse_pos: Vector2,
    window_events: Vec<Event>,
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

fn mouse_button(mb: &MouseButton) -> Option<iced::mouse::Button> {
    match mb {
        MouseButton::Left => Some(iced::mouse::Button::Left),
        MouseButton::Right => Some(iced::mouse::Button::Right),
        MouseButton::Middle => Some(iced::mouse::Button::Middle),
        MouseButton::Other(i) => Some(iced::mouse::Button::Other(*i)),
        _ => None,
    }
}


// fuck you
fn conv_key(key: winit::keyboard::Key) -> iced::keyboard::Key {
    use iced::keyboard::key::Named;
    use winit::keyboard::NamedKey;

    match key {
        winit::keyboard::Key::Character(c) => iced::keyboard::Key::Character(c),
        winit::keyboard::Key::Named(named_key) => {
            iced::keyboard::Key::Named(match named_key {
                NamedKey::Alt => Named::Alt,
                NamedKey::AltGraph => Named::AltGraph,
                NamedKey::CapsLock => Named::CapsLock,
                NamedKey::Control => Named::Control,
                NamedKey::Fn => Named::Fn,
                NamedKey::FnLock => Named::FnLock,
                NamedKey::NumLock => Named::NumLock,
                NamedKey::ScrollLock => Named::ScrollLock,
                NamedKey::Shift => Named::Shift,
                NamedKey::Symbol => Named::Symbol,
                NamedKey::SymbolLock => Named::SymbolLock,
                NamedKey::Meta => Named::Meta,
                NamedKey::Hyper => Named::Hyper,
                NamedKey::Super => Named::Super,
                NamedKey::Enter => Named::Enter,
                NamedKey::Tab => Named::Tab,
                NamedKey::Space => Named::Space,
                NamedKey::ArrowDown => Named::ArrowDown,
                NamedKey::ArrowLeft => Named::ArrowLeft,
                NamedKey::ArrowRight => Named::ArrowRight,
                NamedKey::ArrowUp => Named::ArrowUp,
                NamedKey::End => Named::End,
                NamedKey::Home => Named::Home,
                NamedKey::PageDown => Named::PageDown,
                NamedKey::PageUp => Named::PageUp,
                NamedKey::Backspace => Named::Backspace,
                NamedKey::Clear => Named::Clear,
                NamedKey::Copy => Named::Copy,
                NamedKey::CrSel => Named::CrSel,
                NamedKey::Cut => Named::Cut,
                NamedKey::Delete => Named::Delete,
                NamedKey::EraseEof => Named::EraseEof,
                NamedKey::ExSel => Named::ExSel,
                NamedKey::Insert => Named::Insert,
                NamedKey::Paste => Named::Paste,
                NamedKey::Redo => Named::Redo,
                NamedKey::Undo => Named::Undo,
                NamedKey::Accept => Named::Accept,
                NamedKey::Again => Named::Again,
                NamedKey::Attn => Named::Attn,
                NamedKey::Cancel => Named::Cancel,
                NamedKey::ContextMenu => Named::ContextMenu,
                NamedKey::Escape => Named::Escape,
                NamedKey::Execute => Named::Execute,
                NamedKey::Find => Named::Find,
                NamedKey::Help => Named::Help,
                NamedKey::Pause => Named::Pause,
                NamedKey::Play => Named::Play,
                NamedKey::Props => Named::Props,
                NamedKey::Select => Named::Select,
                NamedKey::ZoomIn => Named::ZoomIn,
                NamedKey::ZoomOut => Named::ZoomOut,
                NamedKey::BrightnessDown => Named::BrightnessDown,
                NamedKey::BrightnessUp => Named::BrightnessUp,
                NamedKey::Eject => Named::Eject,
                NamedKey::LogOff => Named::LogOff,
                NamedKey::Power => Named::Power,
                NamedKey::PowerOff => Named::PowerOff,
                NamedKey::PrintScreen => Named::PrintScreen,
                NamedKey::Hibernate => Named::Hibernate,
                NamedKey::Standby => Named::Standby,
                NamedKey::WakeUp => Named::WakeUp,
                NamedKey::AllCandidates => Named::AllCandidates,
                NamedKey::Alphanumeric => Named::Alphanumeric,
                NamedKey::CodeInput => Named::CodeInput,
                NamedKey::Compose => Named::Compose,
                NamedKey::Convert => Named::Convert,
                NamedKey::FinalMode => Named::FinalMode,
                NamedKey::GroupFirst => Named::GroupFirst,
                NamedKey::GroupLast => Named::GroupLast,
                NamedKey::GroupNext => Named::GroupNext,
                NamedKey::GroupPrevious => Named::GroupPrevious,
                NamedKey::ModeChange => Named::ModeChange,
                NamedKey::NextCandidate => Named::NextCandidate,
                NamedKey::NonConvert => Named::NonConvert,
                NamedKey::PreviousCandidate => Named::PreviousCandidate,
                NamedKey::Process => Named::Process,
                NamedKey::SingleCandidate => Named::SingleCandidate,
                NamedKey::HangulMode => Named::HangulMode,
                NamedKey::HanjaMode => Named::HanjaMode,
                NamedKey::JunjaMode => Named::JunjaMode,
                NamedKey::Eisu => Named::Eisu,
                NamedKey::Hankaku => Named::Hankaku,
                NamedKey::Hiragana => Named::Hiragana,
                NamedKey::HiraganaKatakana => Named::HiraganaKatakana,
                NamedKey::KanaMode => Named::KanaMode,
                NamedKey::KanjiMode => Named::KanjiMode,
                NamedKey::Katakana => Named::Katakana,
                NamedKey::Romaji => Named::Romaji,
                NamedKey::Zenkaku => Named::Zenkaku,
                NamedKey::ZenkakuHankaku => Named::ZenkakuHankaku,
                NamedKey::Soft1 => Named::Soft1,
                NamedKey::Soft2 => Named::Soft2,
                NamedKey::Soft3 => Named::Soft3,
                NamedKey::Soft4 => Named::Soft4,
                NamedKey::ChannelDown => Named::ChannelDown,
                NamedKey::ChannelUp => Named::ChannelUp,
                NamedKey::Close => Named::Close,
                NamedKey::MailForward => Named::MailForward,
                NamedKey::MailReply => Named::MailReply,
                NamedKey::MailSend => Named::MailSend,
                NamedKey::MediaClose => Named::MediaClose,
                NamedKey::MediaFastForward => Named::MediaFastForward,
                NamedKey::MediaPause => Named::MediaPause,
                NamedKey::MediaPlay => Named::MediaPlay,
                NamedKey::MediaPlayPause => Named::MediaPlayPause,
                NamedKey::MediaRecord => Named::MediaRecord,
                NamedKey::MediaRewind => Named::MediaRewind,
                NamedKey::MediaStop => Named::MediaStop,
                NamedKey::MediaTrackNext => Named::MediaTrackNext,
                NamedKey::MediaTrackPrevious => Named::MediaTrackPrevious,
                NamedKey::New => Named::New,
                NamedKey::Open => Named::Open,
                NamedKey::Print => Named::Print,
                NamedKey::Save => Named::Save,
                NamedKey::SpellCheck => Named::SpellCheck,
                NamedKey::Key11 => Named::Key11,
                NamedKey::Key12 => Named::Key12,
                NamedKey::AudioBalanceLeft => Named::AudioBalanceLeft,
                NamedKey::AudioBalanceRight => Named::AudioBalanceRight,
                NamedKey::AudioBassBoostDown => Named::AudioBassBoostDown,
                NamedKey::AudioBassBoostToggle => Named::AudioBassBoostToggle,
                NamedKey::AudioBassBoostUp => Named::AudioBassBoostUp,
                NamedKey::AudioFaderFront => Named::AudioFaderFront,
                NamedKey::AudioFaderRear => Named::AudioFaderRear,
                NamedKey::AudioSurroundModeNext => Named::AudioSurroundModeNext,
                NamedKey::AudioTrebleDown => Named::AudioTrebleDown,
                NamedKey::AudioTrebleUp => Named::AudioTrebleUp,
                NamedKey::AudioVolumeDown => Named::AudioVolumeDown,
                NamedKey::AudioVolumeUp => Named::AudioVolumeUp,
                NamedKey::AudioVolumeMute => Named::AudioVolumeMute,
                NamedKey::MicrophoneToggle => Named::MicrophoneToggle,
                NamedKey::MicrophoneVolumeDown => Named::MicrophoneVolumeDown,
                NamedKey::MicrophoneVolumeUp => Named::MicrophoneVolumeUp,
                NamedKey::MicrophoneVolumeMute => Named::MicrophoneVolumeMute,
                NamedKey::SpeechCorrectionList => Named::SpeechCorrectionList,
                NamedKey::SpeechInputToggle => Named::SpeechInputToggle,
                NamedKey::LaunchApplication1 => Named::LaunchApplication1,
                NamedKey::LaunchApplication2 => Named::LaunchApplication2,
                NamedKey::LaunchCalendar => Named::LaunchCalendar,
                NamedKey::LaunchContacts => Named::LaunchContacts,
                NamedKey::LaunchMail => Named::LaunchMail,
                NamedKey::LaunchMediaPlayer => Named::LaunchMediaPlayer,
                NamedKey::LaunchMusicPlayer => Named::LaunchMusicPlayer,
                NamedKey::LaunchPhone => Named::LaunchPhone,
                NamedKey::LaunchScreenSaver => Named::LaunchScreenSaver,
                NamedKey::LaunchSpreadsheet => Named::LaunchSpreadsheet,
                NamedKey::LaunchWebBrowser => Named::LaunchWebBrowser,
                NamedKey::LaunchWebCam => Named::LaunchWebCam,
                NamedKey::LaunchWordProcessor => Named::LaunchWordProcessor,
                NamedKey::BrowserBack => Named::BrowserBack,
                NamedKey::BrowserFavorites => Named::BrowserFavorites,
                NamedKey::BrowserForward => Named::BrowserForward,
                NamedKey::BrowserHome => Named::BrowserHome,
                NamedKey::BrowserRefresh => Named::BrowserRefresh,
                NamedKey::BrowserSearch => Named::BrowserSearch,
                NamedKey::BrowserStop => Named::BrowserStop,
                NamedKey::AppSwitch => Named::AppSwitch,
                NamedKey::Call => Named::Call,
                NamedKey::Camera => Named::Camera,
                NamedKey::CameraFocus => Named::CameraFocus,
                NamedKey::EndCall => Named::EndCall,
                NamedKey::GoBack => Named::GoBack,
                NamedKey::GoHome => Named::GoHome,
                NamedKey::HeadsetHook => Named::HeadsetHook,
                NamedKey::LastNumberRedial => Named::LastNumberRedial,
                NamedKey::Notification => Named::Notification,
                NamedKey::MannerMode => Named::MannerMode,
                NamedKey::VoiceDial => Named::VoiceDial,
                NamedKey::TV => Named::TV,
                NamedKey::TV3DMode => Named::TV3DMode,
                NamedKey::TVAntennaCable => Named::TVAntennaCable,
                NamedKey::TVAudioDescription => Named::TVAudioDescription,
                NamedKey::TVAudioDescriptionMixDown => {
                    Named::TVAudioDescriptionMixDown
                }
                NamedKey::TVAudioDescriptionMixUp => {
                    Named::TVAudioDescriptionMixUp
                }
                NamedKey::TVContentsMenu => Named::TVContentsMenu,
                NamedKey::TVDataService => Named::TVDataService,
                NamedKey::TVInput => Named::TVInput,
                NamedKey::TVInputComponent1 => Named::TVInputComponent1,
                NamedKey::TVInputComponent2 => Named::TVInputComponent2,
                NamedKey::TVInputComposite1 => Named::TVInputComposite1,
                NamedKey::TVInputComposite2 => Named::TVInputComposite2,
                NamedKey::TVInputHDMI1 => Named::TVInputHDMI1,
                NamedKey::TVInputHDMI2 => Named::TVInputHDMI2,
                NamedKey::TVInputHDMI3 => Named::TVInputHDMI3,
                NamedKey::TVInputHDMI4 => Named::TVInputHDMI4,
                NamedKey::TVInputVGA1 => Named::TVInputVGA1,
                NamedKey::TVMediaContext => Named::TVMediaContext,
                NamedKey::TVNetwork => Named::TVNetwork,
                NamedKey::TVNumberEntry => Named::TVNumberEntry,
                NamedKey::TVPower => Named::TVPower,
                NamedKey::TVRadioService => Named::TVRadioService,
                NamedKey::TVSatellite => Named::TVSatellite,
                NamedKey::TVSatelliteBS => Named::TVSatelliteBS,
                NamedKey::TVSatelliteCS => Named::TVSatelliteCS,
                NamedKey::TVSatelliteToggle => Named::TVSatelliteToggle,
                NamedKey::TVTerrestrialAnalog => Named::TVTerrestrialAnalog,
                NamedKey::TVTerrestrialDigital => Named::TVTerrestrialDigital,
                NamedKey::TVTimer => Named::TVTimer,
                NamedKey::AVRInput => Named::AVRInput,
                NamedKey::AVRPower => Named::AVRPower,
                NamedKey::ColorF0Red => Named::ColorF0Red,
                NamedKey::ColorF1Green => Named::ColorF1Green,
                NamedKey::ColorF2Yellow => Named::ColorF2Yellow,
                NamedKey::ColorF3Blue => Named::ColorF3Blue,
                NamedKey::ColorF4Grey => Named::ColorF4Grey,
                NamedKey::ColorF5Brown => Named::ColorF5Brown,
                NamedKey::ClosedCaptionToggle => Named::ClosedCaptionToggle,
                NamedKey::Dimmer => Named::Dimmer,
                NamedKey::DisplaySwap => Named::DisplaySwap,
                NamedKey::DVR => Named::DVR,
                NamedKey::Exit => Named::Exit,
                NamedKey::FavoriteClear0 => Named::FavoriteClear0,
                NamedKey::FavoriteClear1 => Named::FavoriteClear1,
                NamedKey::FavoriteClear2 => Named::FavoriteClear2,
                NamedKey::FavoriteClear3 => Named::FavoriteClear3,
                NamedKey::FavoriteRecall0 => Named::FavoriteRecall0,
                NamedKey::FavoriteRecall1 => Named::FavoriteRecall1,
                NamedKey::FavoriteRecall2 => Named::FavoriteRecall2,
                NamedKey::FavoriteRecall3 => Named::FavoriteRecall3,
                NamedKey::FavoriteStore0 => Named::FavoriteStore0,
                NamedKey::FavoriteStore1 => Named::FavoriteStore1,
                NamedKey::FavoriteStore2 => Named::FavoriteStore2,
                NamedKey::FavoriteStore3 => Named::FavoriteStore3,
                NamedKey::Guide => Named::Guide,
                NamedKey::GuideNextDay => Named::GuideNextDay,
                NamedKey::GuidePreviousDay => Named::GuidePreviousDay,
                NamedKey::Info => Named::Info,
                NamedKey::InstantReplay => Named::InstantReplay,
                NamedKey::Link => Named::Link,
                NamedKey::ListProgram => Named::ListProgram,
                NamedKey::LiveContent => Named::LiveContent,
                NamedKey::Lock => Named::Lock,
                NamedKey::MediaApps => Named::MediaApps,
                NamedKey::MediaAudioTrack => Named::MediaAudioTrack,
                NamedKey::MediaLast => Named::MediaLast,
                NamedKey::MediaSkipBackward => Named::MediaSkipBackward,
                NamedKey::MediaSkipForward => Named::MediaSkipForward,
                NamedKey::MediaStepBackward => Named::MediaStepBackward,
                NamedKey::MediaStepForward => Named::MediaStepForward,
                NamedKey::MediaTopMenu => Named::MediaTopMenu,
                NamedKey::NavigateIn => Named::NavigateIn,
                NamedKey::NavigateNext => Named::NavigateNext,
                NamedKey::NavigateOut => Named::NavigateOut,
                NamedKey::NavigatePrevious => Named::NavigatePrevious,
                NamedKey::NextFavoriteChannel => Named::NextFavoriteChannel,
                NamedKey::NextUserProfile => Named::NextUserProfile,
                NamedKey::OnDemand => Named::OnDemand,
                NamedKey::Pairing => Named::Pairing,
                NamedKey::PinPDown => Named::PinPDown,
                NamedKey::PinPMove => Named::PinPMove,
                NamedKey::PinPToggle => Named::PinPToggle,
                NamedKey::PinPUp => Named::PinPUp,
                NamedKey::PlaySpeedDown => Named::PlaySpeedDown,
                NamedKey::PlaySpeedReset => Named::PlaySpeedReset,
                NamedKey::PlaySpeedUp => Named::PlaySpeedUp,
                NamedKey::RandomToggle => Named::RandomToggle,
                NamedKey::RcLowBattery => Named::RcLowBattery,
                NamedKey::RecordSpeedNext => Named::RecordSpeedNext,
                NamedKey::RfBypass => Named::RfBypass,
                NamedKey::ScanChannelsToggle => Named::ScanChannelsToggle,
                NamedKey::ScreenModeNext => Named::ScreenModeNext,
                NamedKey::Settings => Named::Settings,
                NamedKey::SplitScreenToggle => Named::SplitScreenToggle,
                NamedKey::STBInput => Named::STBInput,
                NamedKey::STBPower => Named::STBPower,
                NamedKey::Subtitle => Named::Subtitle,
                NamedKey::Teletext => Named::Teletext,
                NamedKey::VideoModeNext => Named::VideoModeNext,
                NamedKey::Wink => Named::Wink,
                NamedKey::ZoomToggle => Named::ZoomToggle,
                NamedKey::F1 => Named::F1,
                NamedKey::F2 => Named::F2,
                NamedKey::F3 => Named::F3,
                NamedKey::F4 => Named::F4,
                NamedKey::F5 => Named::F5,
                NamedKey::F6 => Named::F6,
                NamedKey::F7 => Named::F7,
                NamedKey::F8 => Named::F8,
                NamedKey::F9 => Named::F9,
                NamedKey::F10 => Named::F10,
                NamedKey::F11 => Named::F11,
                NamedKey::F12 => Named::F12,
                NamedKey::F13 => Named::F13,
                NamedKey::F14 => Named::F14,
                NamedKey::F15 => Named::F15,
                NamedKey::F16 => Named::F16,
                NamedKey::F17 => Named::F17,
                NamedKey::F18 => Named::F18,
                NamedKey::F19 => Named::F19,
                NamedKey::F20 => Named::F20,
                NamedKey::F21 => Named::F21,
                NamedKey::F22 => Named::F22,
                NamedKey::F23 => Named::F23,
                NamedKey::F24 => Named::F24,
                NamedKey::F25 => Named::F25,
                NamedKey::F26 => Named::F26,
                NamedKey::F27 => Named::F27,
                NamedKey::F28 => Named::F28,
                NamedKey::F29 => Named::F29,
                NamedKey::F30 => Named::F30,
                NamedKey::F31 => Named::F31,
                NamedKey::F32 => Named::F32,
                NamedKey::F33 => Named::F33,
                NamedKey::F34 => Named::F34,
                NamedKey::F35 => Named::F35,
                _ => return iced::keyboard::Key::Unidentified,
            })
        }
        _ => iced::keyboard::Key::Unidentified,
    }
}

fn conv_location(location: winit::keyboard::KeyLocation) -> iced::keyboard::Location {
    match location {
        winit::keyboard::KeyLocation::Standard => iced::keyboard::Location::Standard,
        winit::keyboard::KeyLocation::Left => iced::keyboard::Location::Left,
        winit::keyboard::KeyLocation::Right => iced::keyboard::Location::Right,
        winit::keyboard::KeyLocation::Numpad => iced::keyboard::Location::Numpad,
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





