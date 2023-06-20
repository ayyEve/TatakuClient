#[derive(Clone, Debug)]
pub enum MediaControlHelperEvent {
    Play,
    Pause,
    Stop,
    Toggle,

    Next,
    Previous,

    SeekForward,
    SeekBackward,
    SeekForwardBy(f32),
    SeekBackwardBy(f32),
    SetPosition(f32),
    OpenUri(String),
    Raise,
    Quit,
}


#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MediaPlaybackState {
    Playing(f32),
    Paused(f32),
    Stopped,
}