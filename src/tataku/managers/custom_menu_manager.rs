use crate::prelude::*;


pub struct CustomMenuManager {
    menu_list: Vec<(CustomMenuSource, CustomMenu)>,
}
impl CustomMenuManager {
    pub fn new() -> Self {
        Self {
            menu_list: Vec::new(),
        }
    }

    pub fn load_menu(&mut self, path: String, source: CustomMenuSource) -> TatakuResult {
        let mut parser = CustomMenuParser::new()?;
        match parser.load_menu(&path) {
            Ok(menu) => self.menu_list.push((source, menu)),
            Err(e) => error!("error loading custom menu: {path}: {e}"),
        }
        
        Ok(())
    }
    pub fn load_menu_from_bytes(&mut self, bytes: &[u8], name: String, source: CustomMenuSource) -> TatakuResult {
        let mut parser = CustomMenuParser::new()?;
        match parser.load_menu_from_bytes(bytes, &name) {
            Ok(menu) => self.menu_list.push((source, menu)),
            Err(e) => error!("error loading custom menu: {name}: {e}"),
        }
        
        Ok(())
    }


    pub fn get_menu(&self, selector: impl Into<CustomMenuSelector>) -> Option<&CustomMenu> {
        let selector:CustomMenuSelector = selector.into();

        for (src, menu) in self.menu_list.iter().rev() {
            if menu.id == selector.name && src.check(&selector.source) {
                return Some(menu)
            }
        }

        None
    }

    pub fn clear_menus(&mut self, source: CustomMenuSource) -> bool {
        let has_entries = !self.menu_list.is_empty();

        self.menu_list.retain(|(src, _)| src.check(&source));

        has_entries
    }


    pub fn update_values(&self, values: &mut ValueCollection) {
        let menu_names = self.menu_list
            .iter()
            .map(|(_src, m)| m.id.clone())
            .collect::<Vec<_>>();

        values.update_or_insert(
            "global.menu_list", 
            TatakuVariableWriteSource::Game, 
            (TatakuVariableAccess::GameOnly, menu_names),
            || TatakuVariable::new_game(TatakuValue::None)
        );
    }
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

