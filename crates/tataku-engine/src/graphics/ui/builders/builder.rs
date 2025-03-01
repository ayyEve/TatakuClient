use crate::prelude::*;

pub struct SettingsBuilder<'a> {
    pub values: &'a mut dyn Reflect, 
    pub categories: Vec<SettingsCategory>, 

    pub create_empty: Box<dyn Fn() -> Box<dyn Widget>>,
    pub create_text: Box<dyn Fn(TextBuilder) -> Box<dyn Widget>>,
    pub create_button: Box<dyn Fn(ButtonBuilder) -> Box<dyn Widget>>,
    pub create_checkbox: Box<dyn Fn(CheckboxBuilder) -> Box<dyn Widget>>,
    pub create_slider: Box<dyn Fn(SliderBuilder) -> Box<dyn Widget>>,
    pub create_text_input: Box<dyn Fn(TextInputBuilder) -> Box<dyn Widget>>,
    pub create_dropdown: Box<dyn Fn(DropdownBuilder) -> Box<dyn Widget>>,
}
impl SettingsBuilder<'_> {
    pub fn add_item(
        &mut self,
        prop: Box<dyn Widget>,
        val: Box<dyn Widget>,
        name: impl ToString,
    ) {
        let sc = self.categories.last_mut().unwrap();
        sc.properties.push(prop);
        sc.values.push(val);
        sc.names.push(name.to_string())
    }

    pub fn add_category(
        &mut self, 
        category: impl ToString,
    ) {
        self.categories.push(SettingsCategory {
            name: category.to_string(),
            ..Default::default()
        });
    }


    pub fn create_empty(&self) -> Box<dyn Widget> {
        (self.create_empty)()
    }
    pub fn create_text(&self, builder: TextBuilder) -> Box<dyn Widget> {
        (self.create_text)(builder)
    }
    pub fn create_button(&self, builder: ButtonBuilder) -> Box<dyn Widget> {
        (self.create_button)(builder)
    }
    pub fn create_checkbox(&self, builder: CheckboxBuilder) -> Box<dyn Widget> {
        (self.create_checkbox)(builder)
    }
    pub fn create_slider(&self, builder: SliderBuilder) -> Box<dyn Widget> {
        (self.create_slider)(builder)
    }
    pub fn create_text_input(&self, builder: TextInputBuilder) -> Box<dyn Widget> {
        (self.create_text_input)(builder)
    }
    pub fn create_dropdown(&self, builder: DropdownBuilder) -> Box<dyn Widget> {
        (self.create_dropdown)(builder)
    }

}
