use crate::prelude::*;


pub struct CustomMenuManager {
    menu_list: Vec<CustomMenuEntry>,
}
impl CustomMenuManager {
    pub fn new() -> Self {
        Self {
            menu_list: Vec::new(),
        }
    }


    fn load_menu_inner(
        path: Option<String>, 
        name: Option<String>, 
        bytes: Vec<u8>, 
        source: CustomMenuSource
    ) -> TatakuResult<CustomMenuEntry> {
        let mut parser = CustomMenuParser::new()?;
        let name = name.unwrap_or_else(|| path.clone().unwrap_or_default());

        let menu = parser.load_menu_from_bytes(&bytes, &name)?;

        Ok(CustomMenuEntry {
            path,
            source,
            menu,
            bytes,
        })
    }

    pub fn load_menu(&mut self, path: String, source: CustomMenuSource) -> TatakuResult {
        let bytes = std::fs::read(&path)?;

        let menu = Self::load_menu_inner(
            Some(path),
            None,
            bytes,
            source
        )?;

        self.menu_list.push(menu);
        Ok(())

        // let mut parser = CustomMenuParser::new()?;
        // match parser.load_menu_from_bytes(&bytes, &path) {
        //     Ok(menu) => {
        //         self.menu_list.push(CustomMenuEntry {
        //             path: Some(path),
        //             source,
        //             menu,
        //             bytes,
        //         })
        //     }
        //     Err(e) => error!("error loading custom menu: {path}: {e}"),
        // }
        
        // Ok(())
    }
    pub fn load_menu_from_bytes(&mut self, bytes: &[u8], name: String, source: CustomMenuSource) -> TatakuResult {
        
        let menu = Self::load_menu_inner(
            None, 
            Some(name), 
            bytes.to_vec(), 
            source
        )?;

        self.menu_list.push(menu);
        Ok(())
        

        
        // let mut parser = CustomMenuParser::new()?;
        // match parser.load_menu_from_bytes(bytes, &name) {
        //     Ok(menu) => self.menu_list.push(CustomMenuEntry {
        //         path: None,
        //         source,
        //         menu,
        //         bytes: bytes.to_vec(),
        //     }),
        //     Err(e) => error!("error loading custom menu: {name}: {e}"),
        // }
        
        // Ok(())
    }

    pub fn load_menu_from_bytes_and_path(
        &mut self, 
        bytes: &[u8], 
        path: String, 
        source: CustomMenuSource
    ) -> TatakuResult {
        let menu = Self::load_menu_inner(
            Some(path), 
            None, 
            bytes.to_vec(), 
            source
        )?;

        self.menu_list.push(menu);
        Ok(())
    }
    


    pub fn get_menu(&self, selector: impl Into<CustomMenuSelector>) -> Option<&CustomMenu> {
        let selector: CustomMenuSelector = selector.into();

        for CustomMenuEntry { source, menu, .. } in self.menu_list.iter().rev() {
            if menu.id == selector.name && source.check(&selector.source) {
                return Some(menu)
            }
        }

        None
    }

    pub fn reload_menus(&mut self, source: CustomMenuSource) -> bool {
        let mut reloaded = false;

        for i in self.menu_list.iter_mut().filter(|m| !m.source.check(&source) ) {
            let Some(path) = &i.path else { continue };
            let Ok(bytes) = std::fs::read(&path) else { continue };

            match Self::load_menu_inner(
                Some(path.clone()), 
                None, 
                bytes, 
                i.source
            ) {
                Ok(menu) => {
                    reloaded = true;
                    i.menu = menu.menu;
                    i.bytes = menu.bytes;
                }
                Err(e) => {
                    error!("error reloading custom menu {path}: {e:?}")
                }
            }
        }

        reloaded
    }

    pub fn clear_menus(&mut self, source: CustomMenuSource) -> bool {
        let has_entries = !self.menu_list.is_empty();

        self.menu_list.retain(|src| src.source.check(&source));

        has_entries
    }


    pub fn update_values(&self, values: &mut ValueCollection) {
        let menu_names = self.menu_list
            .iter()
            .map(|m| m.menu.id.clone())
            .collect::<Vec<_>>();

        values.update_or_insert(
            "global.menu_list", 
            TatakuVariableWriteSource::Game, 
            (TatakuVariableAccess::GameOnly, menu_names),
            || TatakuVariable::new_game(TatakuValue::None)
        );
    }
}


struct CustomMenuEntry {
    source: CustomMenuSource,
    menu: CustomMenu,
    
    path: Option<String>,
    bytes: Vec<u8>,
}

#[derive(Default)]
pub struct CustomMenuSelector {
    name: String,
    source: CustomMenuSource,
}
impl From<(String, CustomMenuSource)> for CustomMenuSelector {
    fn from((name, source): (String, CustomMenuSource)) -> Self {
        Self {
            name,
            source,
        }
    }
}
impl From<String> for CustomMenuSelector {
    fn from(name: String) -> Self {
        Self {
            name,
            source: CustomMenuSource::Any,
        }
    }
}
impl From<&str> for CustomMenuSelector {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            source: CustomMenuSource::Any,
        }
    }
}


#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum CustomMenuSource {
    /// Will pick the last loaded menu from the list 
    #[default]
    Any,

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

