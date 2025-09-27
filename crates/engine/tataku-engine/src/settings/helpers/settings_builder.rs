use crate::*;
use common::reflect::Reflect;

use settings::buildable_settings_provider::*;

pub struct SettingsBuilder<'a> {
    pub values: &'a mut dyn Reflect,

    data: BuildableSettingsProvider,
    current_category: Option<BuildableSettingsCategory>,
    current_id: u16,
}
impl<'a> SettingsBuilder<'a> {
    pub fn new(values: &'a mut dyn Reflect, name: impl Into<String>) -> Self {
        Self {
            values,
            data: BuildableSettingsProvider {
                name: name.into(),
                ..Default::default()
            },
            current_category: None,
            current_id: 0,
        }
    }
    pub fn done(mut self) -> BuildableSettingsProvider {
        if let Some(last) = self.current_category {
            self.data.categories.push(last);
        }
        self.data
    }

    pub fn add_item(
        &mut self, 
        setting: BuildableSetting,
    ) {
        let Some(category) = &mut self.current_category 
        else { panic!("no current category when adding {}", setting.path) };

        category.settings.push(Arc::new(setting));
    }
    pub fn add_category<T: Into<String>>(
        &mut self, 
        name: impl Into<String>,
        icon: Option<T>,
    ) {
        if let Some(category) = self.current_category.take() {
            self.current_id += 1;
            self.data.categories.push(category);
        }

        self.current_category = Some(BuildableSettingsCategory {
            name: name.into(),
            icon: icon.map(|i| i.into()),
            id: self.current_id,
            settings: Vec::new(),
        });
    }
}
