use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(Reflect)]
pub struct ChatMessage {
    pub sender: String,
    // channel or username
    pub channel: ChatChannelType, 
    pub sender_id: u32,
    pub timestamp: u64, //TODO: make this not shit
    pub text: String,

    pub formatted: String,
}
impl ChatMessage {
    pub fn now() -> u64 {
        match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis() as u64,
            Err(_) => 0,
        }
    }
    pub fn new(sender: String, channel: ChatChannelType, sender_id: u32, text: String) -> Self {
        let timestamp = ChatMessage::now();
        let formatted = Self::get_formatted_text(timestamp, &sender, &text);
        
        Self {
            sender,
            channel,
            sender_id,
            text,
            timestamp,
            formatted,
        }
    }

    fn format_time(timestamp: u64) -> String {
        let hours = (timestamp as f64 / (1000.0 * 60.0 * 60.0)).floor() as u64 % 24;
        let minutes = (timestamp as f64 / (1000.0 * 60.0)).floor() as u64 % 60;
        format!("{hours:02}:{minutes:02}")
    }

    fn get_formatted_text(
        timestamp: u64,
        sender: &String,
        text: &String,
    ) -> String {
        let timestamp = Self::format_time(timestamp);

        format!("{timestamp} {sender}: {text}")
    }
}
