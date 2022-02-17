// std imports
pub use std::path::Path;
pub use std::fmt::Display;
pub use std::f64::consts::PI;
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::time::{Duration, Instant};

// piston imports
pub use piston::Key;
pub use piston::RenderArgs;
pub use piston::MouseButton;

// graphics imports
pub use graphics::CharacterCache;

// tokio imports
pub use tokio::sync::OnceCell;
// serde imports
pub use serde::{Serialize, Deserialize};

// ui imports
pub use ayyeve_piston_ui::menu::*;
pub use ayyeve_piston_ui::menu::menu_elements::*;
pub use ayyeve_piston_ui::render::{Renderable, Vector2, Color};
pub use ayyeve_piston_ui::prelude::{ScrollableItemGettersSetters, ScrollableGettersSetters};

// font things
pub use ayyeve_piston_ui::render::fonts::get_font;
pub type Font = Arc<Mutex<opengl_graphics::GlyphCache<'static>>>;

// tataku-common imports
pub use tataku_common::types::*;

// folder imports
pub use crate::DOWNLOADS_DIR;

// audio imports
#[cfg(feature="bass_audio")]
pub use bass_rs::prelude::*;
#[cfg(feature="neb_audio")]
pub use crate::game::{AudioHandle, Sound};
#[cfg(feature="neb_audio")]
pub use crate::game::audio::fft::*;

// game and helper imports
pub use crate::menu::*;
pub use crate::game::*;
pub use crate::RetainMut;
pub use crate::graphics::*;
pub use crate::game::audio::*;
pub use crate::game::managers::*;
pub use crate::game::helpers::centered_text_helper::CenteredTextHelper;
pub use crate::game::helpers::{*, io::*, math::*, curve::*, key_counter::*, crypto::*};

// sync imports
pub use std::sync::{Arc, Weak};
pub use std::sync::atomic::{*, Ordering::SeqCst};
pub use parking_lot::{Mutex, MutexGuard, RwLock};

// error imports
pub use crate::errors::*;

// gameplay imports
pub use crate::gameplay::*;
pub use crate::gameplay::modes::*;

// beatmap imports
pub use crate::beatmaps::Beatmap;
pub use crate::beatmaps::common::*;
pub use crate::beatmaps::osu::hitobject_defs::*;

// online imports
pub use crate::send_packet;
pub use crate::create_packet;
pub use crate::game::online::*;
pub use tataku_common::PacketId;
pub use tataku_common::serialization::*;

// skin imports
pub use crate::skinning::*;