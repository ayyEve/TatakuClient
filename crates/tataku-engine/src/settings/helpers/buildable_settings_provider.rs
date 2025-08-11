use crate::prelude::*;

#[derive(Clone, Default)]
#[derive(Reflect)]
pub struct BuildableSettingsProvider {
    pub name: String,
    pub categories: Vec<BuildableSettingsCategory>,
}
impl PartialEq for BuildableSettingsProvider {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone)]
#[derive(Reflect)]
#[reflect(display="display")]
pub struct BuildableSettingsCategory {
    pub name: String,
    pub id: u16,
    pub settings: Vec<Arc<BuildableSetting>>,
}
impl Display for BuildableSettingsCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Reflect)]
#[reflect(dont_clone)]
pub struct BuildableSetting {
    pub name: String,
    pub icon: Option<String>,

    // pub id: String,
    pub path: String,
    pub tooltip: Option<String>,

    /// a calc string
    pub enabled_if: Option<String>,
    /// a calc string
    pub visible_if: Option<String>,

    #[reflect(rename="type")]
    pub setting_type: BuildableSettingType,
}

#[derive(Clone)]
#[derive(Reflect)]
#[reflect(display="display")]
pub enum BuildableSettingType {
    // spacing
    Divider,

    /// checkbox
    Bool,

    /// color input
    Color,

    /// key input
    Key {
        optional: bool,
    },

    /// text input
    String {
        password: bool,
        // char_limit: Option<usize>,
    },

    /// slider
    Number {
        num_type: String,
        min: f32,
        max: f32,
        step: Option<f32>,
    },

    Dropdown {
        // allow_unset: bool,
        options: BuildableSettingDropdownOptions,
    },

    Button {
        action: BuildableSettingsAction, //BuildableSettingButtonAction,
    },
}
impl std::fmt::Display for BuildableSettingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.__reflect_variant_name().fmt(f)
    }
}

#[derive(Clone)]
#[derive(Reflect)]
pub enum BuildableSettingDropdownOptions {
    List {
        list: Vec<BuildableSettingDropdownListOption>,
    },
    Variable {
        var: String,
    },
}

#[derive(Clone)]
#[derive(Reflect)]
pub struct BuildableSettingDropdownListOption {
    pub name: String,
    #[reflect(skip)]
    pub value: TatakuValue,
}

// #[derive(Clone)]
// #[derive(Reflect)]
// pub enum BuildableSettingButtonAction {
    
// }


#[derive(Clone)]
#[derive(Reflect)]
pub struct BuildableSettingsAction {
    #[reflect(skip)]
    pub inner: Arc<dyn BuildableSettingsActionTrait>,
}
impl From<TatakuAction> for BuildableSettingsAction {
    fn from(value: TatakuAction) -> Self {
        Self {
            inner: Arc::new(value)
        }
    }
}


#[cfg(not(feature="graphics"))] type NodeId = ();

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
