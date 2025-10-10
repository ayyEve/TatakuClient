use crate::prelude::*;

use interface::{
    CustomMenu,
    CustomDialog,
};

#[derive(Default)]
pub struct CustomMenuManager {
    menu_list: Vec<CustomEntry<CustomMenu>>,
    dialog_list: Vec<CustomEntry<CustomDialog>>,
}
impl CustomMenuManager {
    pub fn load_entry(
        &mut self, 
        data: &str,
        path: Option<String>,
        source: CustomMenuSource,
        entry_type: CustomEntryType,
    ) -> tataku::Result<()> {
        match entry_type {
            CustomEntryType::Menu => {
                let menu = quick_xml::de::from_str(data)
                    .map_err(tataku::Error::from_err)?;

                self.menu_list.push(CustomEntry {
                    path,
                    source,
                    inner: menu,
                });
            }
            CustomEntryType::Dialog => {
                let dialog = quick_xml::de::from_str(data)
                    .map_err(tataku::Error::from_err)?;

                self.dialog_list.push(CustomEntry {
                    path,
                    source,
                    inner: dialog,
                });
            }
        }
        Ok(())
    }

    pub fn reload_entries(
        &mut self, 
        source: CustomMenuSource
    ) -> bool {
        let mut reloaded = false;

        for i in self.menu_list.iter_mut().filter(|m| !m.source.check(&source) ) {
            let Some(path) = &i.path else { continue };
            let Ok(data) = std::fs::read_to_string(path) else { continue };

            match quick_xml::de::from_str(&data) {
                Ok(menu) => {
                    i.inner = menu;

                    reloaded = true;
                },
                Err(e) => {
                    error!("error reloading custom menu {path}: {e:?}");
                }
            }
        }

        for i in self.dialog_list.iter_mut().filter(|m| !m.source.check(&source) ) {
            let Some(path) = &i.path else { continue };
            let Ok(data) = std::fs::read_to_string(path) else { continue };

            match quick_xml::de::from_str(&data) {
                Ok(dialog) => {
                    i.inner = dialog;

                    reloaded = true;
                },
                Err(e) => {
                    error!("error reloading custom dialog {path}: {e:?}");
                }
            }
        }

        reloaded
    }

    pub fn clear(&mut self, source: CustomMenuSource) -> bool {
        let mut has_entries = !self.menu_list.is_empty();
        has_entries |= !self.dialog_list.is_empty();

        self.menu_list.retain(|src| src.source.check(&source));
        self.dialog_list.retain(|src| src.source.check(&source));

        has_entries
    }


    pub fn update_values(&self, values: &mut ValueCollection) {
        values.global.menu_list = self.menu_list
            .iter()
            .map(|m| m.inner.id.clone())
            .collect::<Vec<_>>();

        values.global.dialog_list = self.dialog_list
            .iter()
            .map(|m| m.inner.id.clone())
            .collect::<Vec<_>>();
        
    }
}

// getters 
impl CustomMenuManager {
    pub fn get_menu(
        &self, 
        selector: impl Into<CustomEntrySelector>
    ) -> Option<&CustomMenu> {
        let selector: CustomEntrySelector = selector.into();

        for CustomEntry { source, inner: menu, .. } in self.menu_list.iter().rev() {
            if menu.id == selector.name && source.check(&selector.source) {
                return Some(menu)
            }
        }

        None
    }

    pub fn get_dialog(
        &self, 
        selector: impl Into<CustomEntrySelector>,
    ) -> Option<&CustomDialog> {
        let selector: CustomEntrySelector = selector.into();

        for CustomEntry { source, inner, .. } in self.dialog_list.iter().rev() {
            if inner.id == selector.name && source.check(&selector.source) {
                return Some(inner)
            }
        }

        None
    }

}

#[derive(Copy, Clone, Debug)]
pub enum CustomEntryType {
    Menu,
    Dialog,
}

#[derive(Debug)]
struct CustomEntry<T> {
    source: CustomMenuSource,
    inner: T,
    
    path: Option<String>,
}


#[derive(Default)]
pub struct CustomEntrySelector {
    name: ArcStr,
    source: CustomMenuSource,
}
impl From<(ArcStr, CustomMenuSource)> for CustomEntrySelector {
    fn from((name, source): (ArcStr, CustomMenuSource)) -> Self {
        Self {
            name,
            source,
        }
    }
}
impl From<ArcStr> for CustomEntrySelector {
    fn from(name: ArcStr) -> Self {
        Self {
            name,
            source: CustomMenuSource::Any,
        }
    }
}
impl From<&str> for CustomEntrySelector {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_owned().into(),
            source: CustomMenuSource::Any,
        }
    }
}


#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum CustomMenuSource {
    /// Will pick the last loaded menu from the list 
    #[default] Any,

    /// Will explicitly load the menu from the skin
    Skin,

    /// Will load the menu from the game
    Game,
}
impl CustomMenuSource {
    pub fn check(&self, other: &Self) -> bool {
        // if either are any, return true
        if let Self::Any = other { return true }
        if let Self::Any = self { return true }

        // otherwise, return equality
        self == other
    }
}
