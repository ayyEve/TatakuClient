use crate::prelude::*;

#[cfg(feature="graphics")]
use wgpu::PresentMode;

pub static AVAILABLE_PRESENT_MODES:OnceCell<Vec<Vsync>> = OnceCell::const_new();

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum Vsync {
    AutoVsync,
    #[default]
    AutoNoVsync,
    Fifo,
    FifoRelaxed,
    Immediate,
    Mailbox,
}
impl Vsync {
    pub fn to_okay(self) -> Self {
        if self.is_okay() {
            self
        } else {
            self.get_fallback()
        }
    }
    fn is_okay(&self) -> bool {
        AVAILABLE_PRESENT_MODES.get().unwrap().contains(self)
    }
    fn get_fallback(&self) -> Self {
        match self {
            Vsync::AutoVsync 
            | Vsync::Fifo
            | Vsync::FifoRelaxed
                => Vsync::AutoVsync,

            Vsync::AutoNoVsync 
            | Vsync::Immediate
            | Vsync::Mailbox
                => Vsync::AutoNoVsync,
        }
    }
}

#[cfg(feature="graphics")]
impl Dropdownable2 for Vsync {
    type T = Self;
    fn variants() -> Vec<Self> {
        AVAILABLE_PRESENT_MODES.get().unwrap().clone()
    }
}

impl Display for Vsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AutoVsync => write!(f, "Auto On"),
            Self::AutoNoVsync => write!(f, "Auto Off"),
            Self::Fifo => write!(f, "Fifo"),
            Self::FifoRelaxed => write!(f, "Fifo Relaxed"),
            Self::Immediate => write!(f, "Immediate"),
            Self::Mailbox => write!(f, "Mailbox"),
        }
    }
}


#[cfg(feature="graphics")]
impl Into<PresentMode> for Vsync {
    fn into(self) -> PresentMode {
        match self {
            Vsync::AutoVsync => PresentMode::AutoVsync,
            Vsync::AutoNoVsync => PresentMode::AutoNoVsync,
            Vsync::Fifo => PresentMode::Fifo,
            Vsync::FifoRelaxed => PresentMode::FifoRelaxed,
            Vsync::Immediate => PresentMode::Immediate,
            Vsync::Mailbox => PresentMode::Mailbox,
        }
    }
}
#[cfg(feature="graphics")]
impl From<PresentMode> for Vsync {
    fn from(value: PresentMode) -> Self {
        match value {
            PresentMode::AutoVsync => Vsync::AutoVsync,
            PresentMode::AutoNoVsync => Vsync::AutoNoVsync,
            PresentMode::Fifo => Vsync::Fifo,
            PresentMode::FifoRelaxed => Vsync::FifoRelaxed,
            PresentMode::Immediate => Vsync::Immediate,
            PresentMode::Mailbox => Vsync::Mailbox,
        }
    }
}

/// helper for reading settings files where vsync was a bool
pub fn vsync_reader<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vsync, D::Error> {
    use std::fmt;
    use serde::de::{self, Visitor};
    use Vsync::*;

    struct VsyncReader;
    impl<'de> Visitor<'de> for VsyncReader {
        type Value = Vsync;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("expected bool or vsync")
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(if v { AutoVsync } else { AutoNoVsync })
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            match v {
                "AutoVsync" => Ok(Vsync::AutoVsync),
                "AutoNoVsync" => Ok(Vsync::AutoNoVsync),
                "Fifo" => Ok(Vsync::Fifo),
                "FifoRelaxed" => Ok(Vsync::FifoRelaxed),
                "Immediate" => Ok(Vsync::Immediate),
                "Mailbox" => Ok(Vsync::Mailbox),
                other => Err(de::Error::invalid_value(de::Unexpected::Str(other), &self))
            }
        }
    }

    deserializer.deserialize_any(VsyncReader)
}


#[test]
/// test to make sure the vsync_reader fn works correctly
fn vsync_reader_test() {

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    struct Test {
        #[serde(deserialize_with = "vsync_reader")]
        v: Vsync
    }
    #[derive(Serialize)]
    struct Test2 { v: bool }

    let vsyncs = [
        Vsync::AutoVsync,
        Vsync::AutoNoVsync,
        Vsync::Fifo,
        Vsync::FifoRelaxed,
        Vsync::Immediate,
        Vsync::Mailbox,
    ];

    for v in vsyncs {
        let t = Test {v};
        let s = serde_json::to_string(&t).unwrap();
        let t2:Test = serde_json::from_str(&s).unwrap();
        assert_eq!(t, t2);
    }

    for (v, t) in [(true, Vsync::AutoVsync), (false, Vsync::AutoNoVsync)] {
        let t2 = Test2 {v};
        let s = serde_json::to_string(&t2).unwrap();
        let t2:Test = serde_json::from_str(&s).unwrap();
        let t = Test {v:t};
        assert_eq!(t, t2);
    }
}
