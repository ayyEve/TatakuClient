use crate::prelude::*;

/// helper for colors inside settings
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Reflect)]
#[reflect(from_string = "from_str")]
#[serde(from="Color", into="Color")]
pub struct SettingsColor {
    pub string: String,
    pub color: Color,
    pub valid: bool,
}
impl SettingsColor {
    pub fn update(&mut self, s: String) {
        if let Some(color) = Color::try_from_hex(&s) {
            self.color = color;
            self.valid = true;
        } else {
            self.valid = false;
        }

        self.string = s;
    }
}
impl PartialEq for SettingsColor {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
    }
}
impl std::str::FromStr for SettingsColor {
    type Err = ReflectError<'static>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = Color::try_from_hex(s);
        Ok(Self {
            string: s.to_owned(),
            valid: c.is_some(),
            color: c.unwrap_or_default(),
        })
    }
}

impl From<Color> for SettingsColor {
    fn from(color: Color) -> Self {
        Self {
            string: color.to_hex(),
            color,
            valid: true,
        }
    }
}
impl From<SettingsColor> for Color {
    fn from(value: SettingsColor) -> Self {
        value.color
    }
}
impl Deref for SettingsColor {
    type Target = Color; 
    fn deref(&self) -> &Self::Target {
        &self.color
    }
}
