
use crate::prelude::*;
use tokio::sync::oneshot;
use iced::Event;
use iced_runtime::{ user_interface, UserInterface };

use iced::advanced::widget::Operation; 
use iced_winit::conversion::key as conv_key;

pub type IcedElement = iced::Element<'static, Message, iced::Theme, IcedRenderer>;
pub type IcedOverlay<'a> = iced::overlay::Element<'a, Message, iced::Theme, IcedRenderer>;
pub type IcedOperation = Box<dyn Operation + Send + Sync>;

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
    #[allow(clippy::new_without_default)]
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
        self.current_menu = MenuType::from_menu(&*menu);
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
        // let mut needs_render = true;
        let mut last_menu = String::new();
        let mut last_draw;
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

                    if let Ok(scale) = values.impl_get(ReflectPath::new("settings.ui_scale")) {
                        let scale = scale.as_ref();
                        if let Some(scale) = scale.downcast_ref() {
                            if renderer.ui_scale != *scale {
                                renderer.ui_scale = *scale;
                            }
                        }
                    }

                    // rebuild ui with the new application
                    if force_refresh || rebuild_next || application.menu.get_name() != last_menu {
                        last_menu = application.menu.get_name().to_owned();
                        rebuild_next = false;
                        // needs_render = true;

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
                    // if needs_render {
                        // needs_render = false;

                        ui.draw(
                            &mut renderer, 
                            &iced::Theme::Dark, 
                            &Default::default(), 
                            iced::mouse::Cursor::Available(mouse_pos)
                        );

                        // renderer.with_primitives(|_b, p| p.iter().for_each(|p| group.push_arced(into_renderable(p))));
                        last_draw = renderer.finish();
                        // last_draw.raw_draw = true;
                    // }

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
impl CurrentInputState<'_> {
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
                modified_key: conv_key(key.logical.clone()),
                physical_key: iced_winit::conversion::physical_key(key.physical), //conv_key(key.physical.clone()),
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
