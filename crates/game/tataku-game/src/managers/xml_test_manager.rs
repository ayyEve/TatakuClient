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
        text_layout_contexts: &mut TextLayoutContexts,
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
                    actions,
                    text_layout_contexts,
                ).is_err() {
                    ui_manager.set_root(
                        EmptyWidget::new_boxed(),
                        values,
                        actions,
                        text_layout_contexts,
                    );
                }
            }
        }

    }

    pub fn load_file(
        &mut self,
        path: String,
        ui_manager: &mut UiManager,
        values: &mut ValueCollection,
        actions: &mut ActionQueue,
        text_layout_contexts: &mut TextLayoutContexts,
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
                let menu = custom_menu.build();

                ui_manager.set_root(
                    Box::new(menu),
                    values,
                    actions,
                    text_layout_contexts,
                );
            }
            XmlData::Dialog(custom_dialog) => {
                let dialog = custom_dialog.build();
                ui_manager.set_root(
                    EmptyWidget::new_boxed(),
                    values,
                    actions,
                    text_layout_contexts,
                );
                ui_manager.add_dialog(
                    Box::new(dialog),
                    DialogCreateOptions::default(),
                    values,
                    actions,
                    text_layout_contexts,
                );
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
