use crate::prelude::*;


#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub enum IpcMessage {
    CheckGameOpen,
    GameIsOpen,

    OpenFile(String),
}

