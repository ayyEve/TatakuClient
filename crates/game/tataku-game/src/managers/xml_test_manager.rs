use crate::prelude::*;
use std::time::SystemTime;

#[derive(Default)]
pub(crate) struct XmlTestManager {
    current_file: Option<CurrentFile>,
}
impl XmlTestManager {
    pub fn update(
        &mut self,
        ui_manager: &mut UiManager,
        values: &mut ValueCollection,
        actions: &mut ActionQueue,
    ) {
        if let Some(file) = self.current_file.as_ref() {
            let Ok(new_meta) = std::fs::metadata(&file.path) 
            else {
                self.current_file = None;
                return
            };

            let modified = new_meta.modified().unwrap();
            #[allow(clippy::collapsible_if)]
            if file.modified != modified {
                if self.load_file(
                    file.path.clone(),
                    ui_manager,
                    values,
                    actions
                ).is_err() {
                    ui_manager.set_root(
                        EmptyWidget::new_boxed(), 
                        values, 
                        actions
                    );
                }
            }
        }

    }

    fn handle_error(
        ui_manager: &mut UiManager,
        error: Vec<BuildableInputError>,
        values: &mut ValueCollection,
        actions: &mut ActionQueue,
        loaded_type: &str,
    ) {
        let mut children = error
            .into_iter()
            .map(|e| format!("{}: {}", e.variable, e.error_type))
            .map(TextWidget::new)
            .map(Widget::boxed)
            .collect::<Vec<_>>();

        children.insert(
            0, 
            TextWidget::new(format!("Error creating {loaded_type}:"))
            .boxed()
        );
        
        let thing = Container::new(children)
            // .flex_direction(ui::FlexDirection::Column)
            .boxed();

        ui_manager.set_root(thing, values, actions);
    }

    pub fn load_file(
        &mut self, 
        path: String,
        ui_manager: &mut UiManager,
        values: &mut ValueCollection,
        actions: &mut ActionQueue,
    ) -> TatakuResult<()> {
        info!("loading file: {path}");
        self.current_file = None;
        self.current_file = Some(CurrentFile {
            path: path.clone(),
            modified: std::fs::metadata(&path)?.modified()?,
        });
        
        let bytes = std::fs::read(&path)?;
        let data = std::io::Cursor::new(bytes);
        let thing = quick_xml::de::from_reader::<_, XmlData>(data)
            .inspect_err(|e| error!("{e:?}"))
            .map_err(TatakuError::from_err)?;

        match thing {
            XmlData::Menu(custom_menu) => {
                let mut input = BuildableInputArguments::default();
                for i in custom_menu.inputs.list.iter() {
                    if let Some(test) = i.test_value.clone() {
                        input.insert(i.name.clone(), test);
                    }
                }

                match custom_menu.build(values, input) {
                    Ok(menu) => ui_manager.set_root(
                        Box::new(menu), 
                        values,
                        actions,
                    ),
                    Err(e) => Self::handle_error(
                        ui_manager, 
                        e, 
                        values, 
                        actions,
                        "menu"
                    ),
                }
            }
            XmlData::Dialog(custom_dialog) => {
                let mut input = BuildableInputArguments::default();
                for i in custom_dialog.inputs.list.iter() {
                    if let Some(test) = i.test_value.clone() {
                        input.insert(i.name.clone(), test);
                    }
                }

                match custom_dialog.build(values, input) {
                    Ok(dialog) => {
                        ui_manager.set_root(
                            EmptyWidget::new_boxed(), 
                            values, 
                            actions
                        );
                        ui_manager.add_dialog(
                            Box::new(dialog), 
                            DialogCreateOptions::default(),
                            values, 
                            actions
                        );
                    },
                    Err(e) => Self::handle_error(
                        ui_manager,
                        e, 
                        values, 
                        actions,
                        "dialog"
                    ),
                }
            }
        }

        Ok(())
    }
}


struct CurrentFile {
    path: String,
    modified: SystemTime,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum XmlData {
    Menu(CustomMenu),
    Dialog(CustomDialog),
    // Widget(CustomWidget),
}