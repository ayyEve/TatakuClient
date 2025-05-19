use crate::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
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
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct CustomMenuKeyEvent {
    /// What key?
    #[serde(alias="@key")] pub key: crate::prelude::Key,

    /// Must control be pressed?
    #[serde(alias="@control", default)] pub control: bool,

    /// Must alt be pressed?
    #[serde(alias="@alt", default)] pub alt: bool,

    /// Must shift be pressed?
    #[serde(alias="@shift", default)] pub shift: bool,
}



#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct CustomMenuControllerEvent {
    #[serde(alias= "@button")] pub button: ControllerButton,
}
