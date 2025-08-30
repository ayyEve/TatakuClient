use crate::prelude::*;
use tataku_ui::prelude::*;

#[derive(Reflect)]
#[derive(Clone, Debug, Default)]
pub struct BuildableSettingsProvider {
    pub name: String,
    pub categories: Vec<BuildableSettingsCategory>,
}
impl PartialEq for BuildableSettingsProvider {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub struct BuildableSettingsCategory {
    pub name: String,
    pub icon: Option<String>,
    pub id: u16,
    pub settings: Vec<Arc<BuildableSetting>>,
}
impl Display for BuildableSettingsCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Debug)]
#[derive(Reflect)]
#[reflect(dont_clone)]
pub struct BuildableSetting {
    pub name: String,
    pub path: String,
    pub tooltip: Option<String>,

    /// a calc string
    pub enabled_if: Option<String>,
    /// a calc string
    pub visible_if: Option<String>,

    #[reflect(rename="type")]
    pub setting_type: BuildableSettingType,
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display="display")]
pub enum BuildableSettingType {
    // Spacing
    Divider,

    /// Checkbox
    Bool,

    /// Color input
    Color,

    /// Key input
    Key {
        optional: bool,
    },

    /// Gamepad button input
    GamepadButton {
        optional: bool,
    },

    /// Text input
    String {
        password: bool,
        // char_limit: Option<usize>,
    },

    /// Slider
    Number {
        num_type: String,
        min: f32,
        max: f32,
        step: Option<f32>,
    },

    // A dropdown
    Dropdown {
        // allow_unset: bool,
        options: BuildableSettingDropdownOptions,
    },

    /// A Button
    Button {
        action: BuildableSettingsAction, //BuildableSettingButtonAction,
    },
}
impl std::fmt::Display for BuildableSettingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.__reflect_variant_name().fmt(f)
    }
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
#[reflect(display = "display")]
pub enum BuildableSettingDropdownOptions {
    List {
        list: Vec<BuildableSettingDropdownListOption>,
    },
    Variable {
        var: String,
    },
}
impl Display for BuildableSettingDropdownOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List {..} => "List",
            Self::Variable {..} => "Variable",
        }.fmt(f)
    }
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
pub struct BuildableSettingDropdownListOption {
    pub name: String,
    #[reflect(skip)]
    pub value: TatakuValue,
}

#[derive(Reflect)]
#[derive(Clone, Debug2)]
pub struct BuildableSettingsAction {
    #[reflect(skip)] #[debug(skip)]
    pub inner: Arc<dyn BuildableSettingsActionTrait>,
}
impl From<TatakuAction> for BuildableSettingsAction {
    fn from(value: TatakuAction) -> Self {
        Self {
            inner: Arc::new(value)
        }
    }
}


pub trait BuildableSettingsActionTrait: Send + Sync {
    fn build(
        &self, 
        node: NodeId,
        passed_in: Option<&TatakuValue>,
        values: &dyn Reflect,
    ) -> Option<TatakuAction>;
}
impl BuildableSettingsActionTrait for TatakuAction {
    fn build(
        &self, 
        _: NodeId,
        _: Option<&TatakuValue>,
        _: &dyn Reflect,
    ) -> Option<TatakuAction> {
        Some(self.clone())
    }
}
