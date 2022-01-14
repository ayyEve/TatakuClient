#![allow(dead_code, unused, non_snake_case)]

use futures_util::SinkExt;

use crate::prelude::*;
const INPUT_HEIGHT:f64 = 45.0;

lazy_static::lazy_static! {
    static ref CHAT_EXISTS:AtomicBool = AtomicBool::new(false);
}

pub struct Chat {
    // messages
    messages: HashMap<ChatChannel, Vec<ChatMessage>>,
    // if the chat is visible or not
    should_close: bool,

    // scrollables
    channel_scroll: ScrollableArea,
    message_scroll: ScrollableArea,
    input: TextInput,

    pub selected_channel: Option<ChatChannel>,

    // TODO: unread messages

    // sizes
    pub chat_height: f64,
    pub channel_list_width: f64,
}
impl Chat {
    pub fn new() -> Option<Self> {
        if CHAT_EXISTS.load(SeqCst) {
            println!("chat exists, not creating");
            return None
        }
        CHAT_EXISTS.store(true, SeqCst);

        let window_size = Settings::window_size();

        let chat_height = window_size.y / 3.0 - INPUT_HEIGHT;
        let channel_list_width = window_size.x / 5.0;


        let chat_size = Vector2::new(window_size.x - channel_list_width, chat_height);
        let chat_pos  = Vector2::new(channel_list_width, window_size.y - chat_height);
        let channel_list_size = Vector2::new(channel_list_width, chat_size.y);

        let mut input = TextInput::new(
            Vector2::new(channel_list_width, window_size.y - INPUT_HEIGHT), 
            Vector2::new(chat_size.x, INPUT_HEIGHT), 
            "Chat: ", 
            ""
        );
        
        Some(Self {
            // [channels][messages]
            messages:HashMap::new(),
            selected_channel: None,
            should_close: false,

            channel_scroll: ScrollableArea::new(Vector2::new(0.0, chat_pos.y), channel_list_size, true),
            message_scroll: ScrollableArea::new(chat_pos, chat_size - Vector2::new(0.0, INPUT_HEIGHT), true),
            input,

            // positions/sizes
            chat_height,
            channel_list_width
        })
    }

    pub fn scroll_to_new_message(&mut self) {
        // make the message scroll think the mouse is on it
        self.message_scroll.on_mouse_move(self.message_scroll.get_pos() + Vector2::one());

        // do a negative max scroll
        self.message_scroll.on_scroll(-f64::MAX);
    }
}
impl Dialog<Game> for Chat {
    fn get_bounds(&self) -> Rectangle {
        let window_size = Settings::window_size();
        Rectangle::bounds_only(
            Vector2::new(0.0, window_size.y - self.chat_height), 
            Vector2::new(
                window_size.x,
                self.chat_height
            )
        )
    }
    fn should_close(&self) -> bool {self.should_close}

    fn on_key_press(&mut self, key:&Key, mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {
            self.should_close = true;
            return true;
        }

        if key == &Key::Return {
            let send_text = self.input.get_text();
            self.input.set_text(String::new());

            if let Some(channel) = self.selected_channel.clone() {
                let clone = ONLINE_MANAGER.clone();
                tokio::spawn(async move {
                    let man = clone.lock().await;
                    send_packet!(man.writer, create_packet!(PacketId::Client_SendMessage {
                        channel: channel.get_name(),
                        message: send_text
                    }));
                });
            }
            return true;
        }

        self.input.on_key_press(*key, *mods);

        true
    }
    fn on_key_release(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        self.input.on_key_release(*key);
        true
    }
    fn on_text(&mut self, text:&String) -> bool {
        self.input.on_text(text.to_owned());
        true
    }

    fn on_mouse_down(&mut self, pos:&Vector2, button:&MouseButton, mods:&KeyModifiers, _g:&mut Game) -> bool {
        println!("click");
        // check if a channel was clicked
        if let Some(channel_name) = self.channel_scroll.on_click_tagged(*pos, *button, *mods) {
            println!("channel clicked: {}", channel_name);

            // find the channel name in the list
            for (channel, message_list) in self.messages.iter() {
                if channel.get_name() != channel_name {continue}

                // set our current channel
                self.selected_channel = Some(channel.clone());

                // clear old messages
                self.message_scroll.clear();

                let window_size = Settings::window_size();
                for m in message_list.iter() {
                    self.message_scroll.add_item(Box::new(MessageScroll::new(
                        m.clone(),
                        window_size.x - self.channel_list_width,
                        30
                    )));
                }
            }

            // scroll to the bottom
            self.scroll_to_new_message();

            return true;
        }

        self.input.on_click(*pos, *button, *mods);
        //TODO: check messages click?


        true
    }

    fn on_mouse_move(&mut self, pos:&Vector2, _g:&mut Game) {
        self.channel_scroll.on_mouse_move(*pos);
        self.message_scroll.on_mouse_move(*pos);
    }

    fn on_mouse_scroll(&mut self, delta:&f64, _g:&mut Game) -> bool {
        self.channel_scroll.on_scroll(*delta);
        self.message_scroll.on_scroll(*delta);

        true
    }


    fn update(&mut self, _g:&mut Game) {
        if let Ok(online_manager) = ONLINE_MANAGER.try_lock() {
            let mut scroll_pending = false;
            // get chat messages
            for (channel, messages) in online_manager.chat_messages.iter() {
                if !self.messages.contains_key(channel) {
                    self.messages.insert(channel.clone(), messages.clone());
                    self.channel_scroll.add_item(Box::new(ChannelScroll::new(
                        channel.clone(), 
                        self.channel_list_width, 
                        30
                    )));
                    continue;
                }

                // update the messages list if there was a new message in the currently selected channel
                if let Some(current_channel) = &self.selected_channel {
                    if channel.get_name() == current_channel.get_name() {
                        let cached_messages = self.messages.get_mut(channel).unwrap();

                        let window_size = Settings::window_size();
                        for message in online_manager.chat_messages.get(channel).unwrap() {
                            if !cached_messages.contains(message) {
                                // cached_messages.push(message.clone())
                                self.message_scroll.add_item(Box::new(MessageScroll::new(
                                    message.clone(),
                                    window_size.x - self.channel_list_width,
                                    30
                                )));
                                scroll_pending = true;
                            }
                        }
                    }
                    
                }
            }

            if scroll_pending {
                // scroll to the bottom
                self.scroll_to_new_message();
            }

            self.messages = online_manager.chat_messages.clone();
        }

        // ensure input is always accepting input
        self.input.set_selected(true);
    }

    fn draw(&mut self, args:&piston::RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        let args = *args;
        let depth = *depth;

        // draw backgrounds
        list.push(Box::new(Rectangle::new(
            Color::WHITE.alpha(0.85),
            depth + 0.005,
            self.channel_scroll.get_pos(),
            self.channel_scroll.size(),
            Some(Border::new(Color::BLACK, 2.0))
        )));
        list.push(Box::new(Rectangle::new(
            Color::WHITE.alpha(0.85),
            depth + 0.005,
            self.message_scroll.get_pos(),
            self.message_scroll.size(), //+ Vector2::new(0.0, INPUT_HEIGHT),
            Some(Border::new(Color::BLACK, 2.0))
        )));

        list.extend(self.channel_scroll.draw(args, Vector2::zero(), depth));
        list.extend(self.message_scroll.draw(args, Vector2::zero(), depth));
        list.extend(self.input.draw(args, Vector2::zero(), depth - 10.0));
    }
}
impl Drop for Chat {
    fn drop(&mut self) {
        CHAT_EXISTS.store(false, SeqCst);
    }
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
        format!("{}:{}", hours, minutes)
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
    pub fn get_name(&self) -> String {
        match self {
            ChatChannel::Channel { name } => format!("#{}", name),
            ChatChannel::User { username } => username.clone(),
        }
    }
}


struct ChannelScroll {
    pos: Vector2,
    size: Vector2,
    hover: bool,
    selected: bool,

    channel: ChatChannel,
    font_size: u32,
    font: Font,
}
impl ChannelScroll {
    fn new(channel: ChatChannel, width: f64, font_size: u32) -> Self {
        Self {
            channel,
            font_size,

            hover: false,
            selected: false,
            pos: Vector2::zero(),
            size: Vector2::new(width, font_size as f64),
            font: get_font("main"),
        }
    }
}
impl ScrollableItem for ChannelScroll {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_tag(&self) -> String {self.channel.get_name()}
    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}

    fn get_selectable(&self) -> bool {true}
    fn get_selected(&self) -> bool {self.selected}
    fn set_selected(&mut self, selected:bool) {self.selected = selected}

    fn draw(&mut self, args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();

        let text = Text::new(
            if self.hover {Color::RED} else if self.selected {Color::BLUE} else {Color::BLACK},
            parent_depth,
            self.pos + pos_offset,
            self.font_size,
            self.channel.get_name(),
            self.font.clone()
        );
        list.push(Box::new(text));
        list
    }
}


struct MessageScroll {
    pos: Vector2,
    size: Vector2,
    hover: bool,

    message: ChatMessage,
    font_size: u32,
    font: Font,
}
impl MessageScroll {
    fn new(message: ChatMessage, width: f64, font_size: u32) -> Self {
        Self {
            message,
            font_size,

            hover: false,
            pos: Vector2::zero(),
            size: Vector2::new(width, font_size as f64),
            font: get_font("main"),
        }
    }
}
impl ScrollableItem for MessageScroll {
    fn size(&self) -> Vector2 {self.size}
    fn get_pos(&self) -> Vector2 {self.pos}
    fn set_pos(&mut self, pos:Vector2) {self.pos = pos}
    fn get_hover(&self) -> bool {self.hover}
    fn set_hover(&mut self, hover:bool) {self.hover = hover}
    // fn get_tag(&self) -> String {self.channel.get_name()}

    fn draw(&mut self, args:RenderArgs, pos_offset:Vector2, parent_depth:f64) -> Vec<Box<dyn Renderable>> {
        let mut list:Vec<Box<dyn Renderable>> = Vec::new();

        let text = Text::new(
            Color::BLACK,
            parent_depth,
            self.pos + pos_offset,
            self.font_size,
            self.message.get_formatted_text(),
            self.font.clone()
        );
        list.push(Box::new(text));
        list
    }
}