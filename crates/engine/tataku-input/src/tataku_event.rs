use crate::prelude::*;

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all="camelCase")]
pub enum TatakuEventType {
    /// Song has ended
    SongEnd,

    /// Song was paused
    SongPause,

    /// Song has started
    SongStart,

    /// Menu was entered
    MenuEnter,
    
    // Menu was left
    MenuLeave,

    /// A new beatmap has been added
    MapAdded,

    /// A key press
    KeyPress(CustomMenuKeyEvent),

    /// A key release
    KeyRelease(CustomMenuKeyEvent),

    /// A controller button was pressed
    ControllerPress(CustomMenuControllerEvent),

    /// A controller button was released
    ControllerRelease(CustomMenuControllerEvent),

    /// A custom event
    #[serde(alias="custom")]
    CustomEvent(String)
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CustomMenuKeyEvent {
    /// What key?
    #[serde(rename="@key")] pub key: Key,

    /// Must control be pressed?
    #[serde(rename="@control", alias="@ctrl", default)] pub control: bool,

    /// Must alt be pressed?
    #[serde(rename="@alt", default)] pub alt: bool,

    /// Must shift be pressed?
    #[serde(rename="@shift", default)] pub shift: bool,
}

#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CustomMenuControllerEvent {
    #[serde(alias= "@button")] pub button: ControllerButton,
}
