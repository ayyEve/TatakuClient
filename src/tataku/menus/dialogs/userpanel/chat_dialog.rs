#![allow(dead_code, unused, non_snake_case)]
use futures_util::SinkExt;
use crate::prelude::*;

//TODO: proper window size

const INPUT_HEIGHT:f32 = 45.0;

const INPUT_FONT_SIZE: f32 = INPUT_HEIGHT * 0.8;

/// how many pixels away from the thing can it be to resize?
const RESIZE_LENIENCE:f32 = 3.0;


pub struct Chat {
    num: usize,

    current_message: String,
    // key_handler: KeyEventsHandlerGroup<ChatDialogKeys>,


    // messages
    messages: HashMap<ChatChannel, Vec<ChatMessage>>,
    // if the chat is visible or not
    should_close: bool,

    // // scrollables
    // channel_scroll: ScrollableArea,
    // message_scroll: ScrollableArea,
    // input: TextInput,

    pub selected_channel: Option<ChatChannel>,

    // TODO: unread messages

    // sizes
    pub chat_height: f32,
    pub channel_list_width: f32,

    // resizing helpers
    width_resize: bool,
    height_resize: bool,
    width_resize_hover: bool,
    height_resize_hover: bool,

    window_size: Arc<WindowSize>,
}
impl Chat {
    pub fn new() -> Self {
        let window_size = WindowSize::get();

        let chat_height = window_size.y / 3.0 - INPUT_HEIGHT;
        let channel_list_width = window_size.x / 5.0;


        let chat_size = Vector2::new(window_size.x - channel_list_width, chat_height);
        let chat_pos  = Vector2::new(channel_list_width, window_size.y - chat_height);
        let channel_list_size = Vector2::new(channel_list_width, chat_size.y);

        // let mut input = TextInput::new(
        //     Vector2::new(channel_list_width, window_size.y - INPUT_HEIGHT), 
        //     Vector2::new(chat_size.x, INPUT_HEIGHT), 
        //     "Chat: ", 
        //     "",
        //     Font::Main,
        // );
        
        Self {
            num: 0,

            // [channels][messages]
            messages:HashMap::new(),
            selected_channel: None,
            should_close: false,
            current_message: String::new(),
            
            // key_handler: KeyEventsHandlerGroup::new(),

            // channel_scroll: ScrollableArea::new(Vector2::new(0.0, chat_pos.y), channel_list_size, ListMode::VerticalList),
            // message_scroll: ScrollableArea::new(chat_pos, chat_size - Vector2::new(0.0, INPUT_HEIGHT), ListMode::VerticalList),
            // input,

            // positions/sizes
            chat_height,
            channel_list_width,

            width_resize:  false,
            height_resize: false,
            width_resize_hover:  false,
            height_resize_hover: false,
            window_size
        }
    }

    pub fn scroll_to_new_message(&mut self) {
        // // make the message scroll think the mouse is on it
        // self.message_scroll.on_mouse_move(self.message_scroll.get_pos() + Vector2::ONE);

        // // do a negative max scroll
        // self.message_scroll.on_scroll(-f32::MAX);
    }
}

#[async_trait]
impl Dialog for Chat {
    fn name(&self) -> &'static str { "chat_dialog" }
    fn get_num(&self) -> usize { self.num }
    fn set_num(&mut self, num: usize) { self.num = num }


    fn should_close(&self) -> bool { self.should_close }
    async fn force_close(&mut self) { self.should_close = true; }

    
    async fn handle_message(&mut self, message: Message, values: &mut ShuntingYardValues) {
        let Some(tag) = message.tag.as_string() else { return }; 

        match &*tag {
            // a channel was clicked
            "channel" => {
                let Some(channel_name) = message.message_type.as_text() else { return }; 
                
                // find the channel name in the list
                for (channel, message_list) in self.messages.iter() {
                    if channel.get_name() != channel_name { continue }

                    // set our current channel
                    self.selected_channel = Some(channel.clone());

                    // // clear old messages
                    // self.message_scroll.clear();

                    // for m in message_list.iter() {
                    //     self.message_scroll.add_item(Box::new(MessageScroll::new(
                    //         m.clone(),
                    //         self.window_size.x - self.channel_list_width,
                    //         30.0
                    //     )));
                    // }
                }
            }
            
            _ => {}
        }
    }

    
    async fn update(&mut self) -> Vec<MenuAction> { 
        // get new messages
        if let Some(mut online_manager) = OnlineManager::try_get_mut() {
            let mut scroll_pending = false;

            if let Some(selected_channel) = &self.selected_channel {
                if !online_manager.chat_messages.contains_key(selected_channel) {
                    online_manager.chat_messages.insert(selected_channel.clone(), Vec::new());
                }

                // ensure the selected channel is actually selected
                let selected_name = selected_channel.get_name();
                // for i in self.channel_scroll.items.iter_mut() {
                //     if i.get_selected() && i.get_tag() != selected_name {
                //         i.set_selected(false)
                //     }

                //     if !i.get_selected() && i.get_tag() == selected_name {
                //         i.set_selected(true)
                //     }
                // }

            }

            // get chat messages
            for (channel, messages) in online_manager.chat_messages.iter() {
                if !self.messages.contains_key(channel) {
                    self.messages.insert(channel.clone(), messages.clone());
                    // self.channel_scroll.add_item(Box::new(ChannelScroll::new(
                    //     channel.clone(), 
                    //     self.channel_list_width, 
                    //     30.0
                    // )));
                    continue;
                }

                // update the messages list if there was a new message in the currently selected channel
                if let Some(current_channel) = &self.selected_channel {
                    if channel.get_name() == current_channel.get_name() {
                        let cached_messages = self.messages.get_mut(channel).unwrap();

                        let window_size = self.window_size.0;
                        for message in online_manager.chat_messages.get(channel).unwrap() {
                            if !cached_messages.contains(message) {
                                // cached_messages.push(message.clone())
                                // self.message_scroll.add_item(Box::new(MessageScroll::new(
                                //     message.clone(),
                                //     window_size.x - self.channel_list_width,
                                //     30.0
                                // )));
                                scroll_pending = true;
                            }
                        }
                    }
                }
            }


            // scroll to the bottom
            if scroll_pending {
                self.scroll_to_new_message();
            }

            self.messages = online_manager.chat_messages.clone();
        }

        // // handle key presses
        // while let Some(key_event) = self.key_handler.check_events() {
        //     match key_event {
        //         KeyEvent::Char(c) => self.current_message.push(c),

        //         KeyEvent::Press(ChatDialogKeys::SendMessage) => {
        //             let send_text = self.current_message.take();

        //             if let Some(channel) = self.selected_channel.clone() {
        //                 tokio::spawn(async move {
        //                     OnlineManager::get().await.send_packet(ChatPacket::Client_SendMessage {
        //                         channel: channel.get_name(),
        //                         message: send_text
        //                     });
        //                 });
        //             }
        //         }

        //         _ => {}
        //     }
        // }


        Vec::new()
    }


    fn view(&self) -> IcedElement {
        use iced_elements::*;
        
        col!(
            
            // channel scroll
            make_scrollable(
                self.messages.keys().map(|c|Text::new(c.get_name()).size(30.0).width(Fill).into_element()).collect(),
                "channel_scroll"
            ),

            // message scroll
            self.selected_channel.as_ref().map(|c|make_scrollable(
                self.messages.get(c).unwrap().iter().map(|c|Text::new(c.text.clone()).size(30.0).width(Fill).into_element()).collect(),
                "message_scroll"
            ).into_element()).unwrap_or_else(||EmptyElement.into_element()),

            // message text input
            TextInput::new("Chat:", &self.current_message).size(INPUT_FONT_SIZE)
            ;
            // // key input
            // self.key_handler.handler();

            width = Fill
        )
    }




    // async fn on_mouse_move(&mut self, pos:Vector2, _g:&mut Game) {
    //     self.channel_scroll.on_mouse_move(pos);
    //     self.message_scroll.on_mouse_move(pos);

    //     let window_size = self.window_size.0;
    //     // self.width_resize_hover = (pos.x - (self.channel_list_width)).powi(2) < RESIZE_LENIENCE.powi(2);
    //     self.height_resize_hover = (pos.y - (window_size.y - self.chat_height)).powi(2) < RESIZE_LENIENCE.powi(2);

    //     if self.height_resize {
    //         self.chat_height = window_size.y - pos.y;

    //         self.channel_scroll.set_pos(Vector2::new(
    //             self.channel_scroll.get_pos().x,
    //             window_size.y - self.chat_height
    //         ));
    //         self.channel_scroll.set_size(Vector2::new(
    //             self.channel_scroll.size().x,
    //             self.chat_height
    //         ));

    //         self.message_scroll.set_pos(Vector2::new(
    //             self.message_scroll.get_pos().x,
    //             window_size.y - self.chat_height
    //         ));
    //         self.message_scroll.set_size(Vector2::new(
    //             self.message_scroll.size().x,
    //             self.chat_height - INPUT_HEIGHT
    //         ));
    //     }
    //     if self.width_resize {
    //         self.channel_list_width = pos.x;

    //         self.channel_scroll.set_size(Vector2::new(
    //             self.channel_list_width,
    //             self.channel_scroll.size().y
    //         ));

    //         self.input.set_pos(Vector2::new(
    //             self.channel_list_width,
    //             self.input.get_pos().y
    //         ));
    //         self.message_scroll.set_pos(Vector2::new(
    //             self.channel_list_width,
    //             self.message_scroll.get_pos().y
    //         ));
    //         self.message_scroll.set_size(Vector2::new(
    //             window_size.x - self.channel_list_width,
    //             self.message_scroll.size().x
    //         ));
    //     }
    // }

    // async fn draw(&mut self, offset: Vector2, list: &mut RenderableCollection) {
    //     let window_size = self.window_size.0;

    //     // draw backgrounds
    //     list.push(Rectangle::new(
    //         self.channel_scroll.get_pos() + offset,
    //         self.channel_scroll.size(),
    //         Color::WHITE.alpha(0.85),
    //         Some(Border::new(Color::BLACK, 2.0))
    //     ));
    //     list.push(Rectangle::new(
    //         self.message_scroll.get_pos() + offset,
    //         self.message_scroll.size(), //+ Vector2::new(0.0, INPUT_HEIGHT),
    //         Color::WHITE.alpha(0.85),
    //         Some(Border::new(Color::BLACK, 2.0))
    //     ));

    //     if self.width_resize_hover {
    //         // red line at width
    //         list.push(Line::new(
    //             Vector2::new(self.channel_list_width, window_size.y) + offset,
    //             Vector2::new(self.channel_list_width, window_size.y - self.chat_height) + offset,
    //             2.0,
    //             Color::RED
    //         ))
    //     }
    //     if self.height_resize_hover {
    //         // red line at height
    //         list.push(Line::new(
    //             Vector2::new(0.0, window_size.y - self.chat_height) + offset,
    //             Vector2::new(window_size.x, window_size.y - self.chat_height) + offset,
    //             2.0,
    //             Color::RED
    //         ))
    //     }

    //     self.channel_scroll.draw(offset, list);
    //     self.message_scroll.draw(offset, list);
    //     self.input.draw(offset, list);
    // }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    pub sender: String,
    // channel or username
    pub channel: ChatChannel, 
    pub sender_id: u32,
    pub timestamp: u64, //TODO: make this not shit
    pub text: String
}
impl ChatMessage {
    pub fn now() -> u64 {
        match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis() as u64,
            Err(_) => 0,
        }
    }
    pub fn new(sender: String, channel: ChatChannel, sender_id: u32, text: String) -> Self {
        Self {
            sender,
            channel,
            sender_id,
            text,
            timestamp: ChatMessage::now()
        }
    }

    pub fn format_time(&self) -> String {
        let hours = (self.timestamp as f64 / (1000.0 * 60.0 * 60.0)).floor() as u64 % 24;
        let minutes = (self.timestamp as f64 / (1000.0 * 60.0)).floor() as u64 % 60;
        format!("{:02}:{:02}", hours, minutes)
    }

    pub fn get_formatted_text(&self) -> String {
        let timestamp = self.format_time();

        format!(
            "{} {}: {}",
            timestamp,
            self.sender,
            self.text
        )
    }
}

// some kind of identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChatChannel {
    Channel{name:String},
    User{username:String}
}
impl ChatChannel {
    pub fn from_name(name:String) -> ChatChannel {
        if name.starts_with("#") {
            ChatChannel::Channel{name}
        } else {
            ChatChannel::User{username: name}
        }
    }
    pub fn get_name(&self) -> String {
        match self {
            ChatChannel::Channel { name } => format!("#{}", name),
            ChatChannel::User { username } => username.clone(),
        }
    }
}



// #[derive(Copy, Clone, Debug)]
// pub enum ChatDialogKeys {
//     SendMessage,
// }

// impl KeyMap for ChatDialogKeys {
//     fn handle_chars() -> bool { true }

//     fn from_key(key: iced::keyboard::KeyCode, mods: iced::keyboard::Modifiers) -> Option<Self> {
//         use iced::keyboard::KeyCode;

//         match key {
//             KeyCode::Enter => Some(Self::SendMessage),

//             _ => None
//         }
//     }
// }






#[derive(ScrollableGettersSetters)]
#[Scrollable(selectable)]
struct ChannelScroll {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,
    tag: String,

    channel: ChatChannel,
    font_size: f32,
    font: Font,
}
impl ChannelScroll {
    fn new(channel: ChatChannel, width: f32, font_size: f32) -> Self {
        Self {
            tag: channel.get_name(),
            channel,
            font_size,

            hover: false,
            selected: false,
            pos: Vector2::ZERO,
            size: Vector2::new(width, font_size),
            font: Font::Main,
        }
    }
}
impl ScrollableItem for ChannelScroll {
    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {

        let text = Text::new(
            self.pos + pos_offset,
            self.font_size,
            self.channel.get_name(),
            if self.hover {Color::RED} else if self.selected {Color::BLUE} else {Color::BLACK},
            self.font.clone()
        );
        list.push(text);
    }
}


#[derive(ScrollableGettersSetters)]
struct MessageScroll {
    pos: Vector2,
    size: Vector2,
    hover: bool,

    message: ChatMessage,
    font_size: f32,
    font: Font,
}
impl MessageScroll {
    fn new(message: ChatMessage, width: f32, font_size: f32) -> Self {
        Self {
            message,
            font_size,

            hover: false,
            pos: Vector2::ZERO,
            size: Vector2::new(width, font_size),
            font: Font::Main,
        }
    }
}
impl ScrollableItem for MessageScroll {
    fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection) {
        let text = Text::new(
            self.pos + pos_offset,
            self.font_size,
            self.message.get_formatted_text(),
            Color::BLACK,
            self.font.clone()
        );
        list.push(text);
    }
}



