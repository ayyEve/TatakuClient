#![allow(unused)]

// TODO: i dont actually know how this works, i thought it was a socket file you can provide but apparently not?
use ipc_channel::ipc::*;
use tokio::sync::mpsc::{ 
    UnboundedSender, 
    UnboundedReceiver, 
    unbounded_channel,
    error::TryRecvError,
};
use serde::{ Serialize, Deserialize };
use tataku_engine::{
    CowStr,
    ArcStr,
    tataku,
    actions,
    Settings,
    tataku::open_link,
    TatakuIntegrationEvent,
    common::reflect::Reflect,
    io::{
        TatakuIntegration,
        TatakuIntegrationBuilder
    },
};
pub struct IpcIntegration {
    sender: UnboundedSender<IpcMessage>,
    receiver: UnboundedReceiver<IpcMessage>,
}
impl IpcIntegration {

    // FIXME: This is some absolute horrible shit and i'll go to hell for writing it
    fn build() -> tataku::Result<Box<dyn TatakuIntegration>> {
        let (sender2, receiver) = unbounded_channel();
        let (sender, mut receiver2) = unbounded_channel();

        let (ipc_sender, ipc_receiver) = ipc_channel::ipc::channel::<IpcMessage>()
            .map_err(tataku::Error::from_err)?;
        std::thread::spawn(move || {
            while let Ok(message) = ipc_receiver.recv() {
                sender2.send(message).unwrap();
            }
        });

        std::thread::spawn(move || {
            loop {
                match receiver2.try_recv() {
                    Ok(message) => ipc_sender.send(message).unwrap(),
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => return,
                }
            }
        });

        Ok(Box::new(Self {
            sender,
            receiver,
        }))
    }
}

#[allow(clippy::single_match)]
impl TatakuIntegration for IpcIntegration {
    fn name(&self) -> CowStr { "ipc_integration".into() }
    fn check_enabled(
        &mut self, 
        _settings: &Settings
    ) -> tataku::Result<()> {
        Ok(())
    }
    
    fn update(
        &mut self, 
        _values: &mut dyn Reflect, 
        _actions: &mut actions::ActionQueue,
    ) {
        let Ok(message) = self.receiver.try_recv() else { return };

        match message {
            // IpcMessage::In_OpenFile(path) => actions.push(GameAction::OpenFile(path)),
            IpcMessage::In_GetValue(_path) => {

            },

            _ => {}
        }
    }
}


#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
pub enum IpcMessage {
    In_OpenFile(String),
    In_GetValue(String),

    Out_ValueResponse(String),
}
