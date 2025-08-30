use crate::prelude::*;
use serde::de::DeserializeOwned;

#[derive(Default)]
pub struct CustomMenuManager {
    menu_list: Vec<CustomEntry<CustomMenu>>,
    dialog_list: Vec<CustomEntry<CustomDialog>>,
}
impl CustomMenuManager {
    fn load_entry_inner<T: DeserializeOwned>(
        path: Option<String>, 
        bytes: Vec<u8>, 
        source: CustomMenuSource
    ) -> TatakuResult<CustomEntry<T>> {
        let menu = quick_xml::de::from_reader(std::io::Cursor::new(&bytes))
            .map_err(TatakuError::from_err)?;

        Ok(CustomEntry {
            path,
            source,
            inner: menu,
            bytes,
        })
    }
    
    pub fn load_entry(
        &mut self, 
        path: String, 
        source: CustomMenuSource,
        entry_type: CustomEntryType,
    ) -> TatakuResult {
        let bytes = std::fs::read(&path)?;
        self.load_entry_bytes(&bytes, Some(path), source, entry_type)
    }
    
    pub fn load_entry_bytes(
        &mut self, 
        bytes: &[u8], 
        path: Option<String>,
        source: CustomMenuSource,
        entry_type: CustomEntryType,
    ) -> TatakuResult {
        match entry_type {
            CustomEntryType::Menu => {
                self.menu_list.push(Self::load_entry_inner(
                    path,
                    bytes.to_vec(),
                    source
                )?);
            }
            CustomEntryType::Dialog => {
                self.dialog_list.push(Self::load_entry_inner(
                    path,
                    bytes.to_vec(),
                    source
                )?);
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
            let Ok(bytes) = std::fs::read(path) else { continue };

            match Self::load_entry_inner(
                Some(path.clone()), 
                bytes, 
                i.source
            ) {
                Ok(menu) => {
                    reloaded = true;
                    i.inner = menu.inner;
                    i.bytes = menu.bytes;
                }
                Err(e) => {
                    error!("error reloading custom menu {path}: {e:?}");
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
    bytes: Vec<u8>,
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
