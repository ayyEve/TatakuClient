use crate::prelude::*;
use tataku::Vector2;

#[derive(Deserialize)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[serde(rename_all="kebab-case")]
pub enum ScrollDirection {
    #[default]
    None,
    Vertical,
    Horizontal,
    Both,
}
impl ScrollDirection {
    pub(super) fn is_some(self) -> bool {
        matches!(self, Self::Horizontal | Self::Vertical | Self::Both)
    }

    pub(super) fn horizontal(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }
    pub(super) fn vertical(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }

    pub fn apply(self, current: &mut Vector2, new: Vector2) -> bool {
        let mut changed = false;
        if self.horizontal() && (current.x - new.x).abs() > f32::EPSILON {
            changed = true;
            current.x = new.x;
        }
        if self.vertical() && (current.y - new.y).abs() > f32::EPSILON {
            changed = true;
            current.y = new.y;
        }

        changed
    }
}
