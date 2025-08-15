use crate::prelude::*;

pub struct SettingsBuilder<'a> {
    pub values: &'a mut dyn Reflect,

    data: BuildableSettingsProvider,
    current_category: Option<BuildableSettingsCategory>,
    current_id: u16,
}
impl<'a> SettingsBuilder<'a> {
    pub fn new(values: &'a mut dyn Reflect, name: impl ToString) -> Self {
        Self {
            values,
            data: BuildableSettingsProvider {
                name: name.to_string(),
                ..Default::default()
            },
            current_category: None,
            current_id: 0,
        }
    }
    pub fn done(self) -> BuildableSettingsProvider {
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
    pub fn add_category<T: ToString>(
        &mut self, 
        name: impl ToString,
        icon: Option<T>,
    ) {
        if let Some(category) = self.current_category.take() {
            self.current_id += 1;
            self.data.categories.push(category);
        }

        self.current_category = Some(BuildableSettingsCategory {
            name: name.to_string(),
            icon: icon.map(|i| i.to_string()),
            id: self.current_id,
            settings: Vec::new(),
        });
    }
}
