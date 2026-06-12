use crate::ElementState;
use tataku_engine_common::common::*;
pub use property_hash::PropertyHashMap;
use crate::style::{
    css::{
        CssStyle,
        CssProperty,
        StyleProperty,
    },
};



#[derive(Clone, Default)]
pub struct PropertyCollection {
    pub id: StyleId<'static>,
    pub enabled: bool,
    pub properties: PropertyHashMap,
}
impl PropertyCollection {
    pub fn from_property_list(
        list: Vec<StyleProperty>,
        id: StyleId<'static>,
    ) -> Self {
        Self {
            id,
            enabled: true,
            properties: list
                .into_iter()
                .map(|i| (i.css_property(), i))
                .collect(),
        }
    }

    // pub fn parse_css(rule: &simplecss::Rule) -> Self {
    //     let list = CssStyle::parse_css(rule)
    //         .into_property_list(true);
    // }
}
impl StylePropertyGroup for PropertyCollection {
    fn id(&self) -> &StyleId { &self.id }
    fn enabled(&self) -> bool { self.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled }

    fn get_property(&self, property: CssProperty) -> Option<Cow<'_, StyleProperty>> {
        self.properties.get(&property).map(Cow::Borrowed)
    }
    fn set_property(&mut self, property: StyleProperty) {
        self.properties.insert(property.css_property(), property);
    }
    fn remove_property(&mut self, property: CssProperty) {
        self.properties.remove(&property);
    }
}


#[derive(Clone, Default)]
pub enum StyleId<'a> {
    #[default]
    Base,
    Overrides,
    NameBorrowed(&'a str),
    Name(String),
    State(ElementState),
}
impl StyleId<'_> {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Base => None,
            Self::Overrides => None,
            Self::State(_) => None,
            Self::Name(s) => Some(s.as_str()),
            Self::NameBorrowed(s) => Some(s),
        }
    }
}
impl From<String> for StyleId<'_> {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}
impl From<ElementState> for StyleId<'_> {
    fn from(value: ElementState) -> Self {
        Self::State(value)
    }
}
impl<'a> From<&'a str> for StyleId<'a> {
    fn from(value: &'a str) -> Self {
        Self::NameBorrowed(value)
    }
}
impl<'a, 'b> PartialEq<StyleId<'a>> for StyleId<'a> {
    fn eq(&self, other: &StyleId<'a>) -> bool {
        if let Some((s1, s2)) = self.name().zip(other.name()) {
            s1 == s2
        } else {
            match (self, other) {
                (Self::Base, Self::Base) => true,

                (Self::State(e1), Self::State(e2)) => {
                    // TODO: is this correct, or should we do some bitchecks?
                    e1 == e2
                }

                _ => false,
            }
        }
    }
}


#[derive(Default)]
pub struct StyleStack {
    cache: PropertyHashMap<Option<StyleProperty>>,

    stack: Vec<Box<dyn StylePropertyGroup>>,
}
impl StyleStack {
    pub fn new<S: StylePropertyGroup + 'static> (base: S) -> Self {
        Self {
            cache: PropertyHashMap::default(),
            stack: vec![ Box::new(base) ]
        }
    }

    pub fn menu_layout() -> Self {
        let list = CssStyle::menu_layout().into_property_list(true);
        Self::new(PropertyCollection::from_property_list(list, StyleId::Base))
    }

    pub fn get(&self, property: CssProperty) -> Option<Cow<'_, StyleProperty>> {
        if let Some(cached) = self.cache.get(&property) {
            cached.as_ref().map(Cow::Borrowed)
        } else {
            self.stack
                .iter().rev()
                .filter(|i| i.enabled())
                .find_map(|i| i.get_property(property))
        }
    }

    pub fn get_group(&mut self, id: StyleId<'_>) -> Option<&mut Box<dyn StylePropertyGroup>> {
        self.stack.iter_mut()
            .find(|i| i.id() == &id)
    }
}


pub trait StylePropertyGroup {
    fn id(&self) -> &StyleId;

    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);

    fn get_property(&self, property: CssProperty) -> Option<Cow<'_, StyleProperty>>;
    fn set_property(&mut self, property: StyleProperty);
    fn remove_property(&mut self, property: CssProperty);

    /// returns if anything was modified
    fn update(&mut self, _time: f32) -> GroupUpdateResponse { GroupUpdateResponse::None }
}

#[derive(Copy, Clone, Debug, Default)]
pub enum GroupUpdateResponse {
    /// Nothing was changed
    #[default]
    None,

    /// An update was made but no relayout required
    Updated,

    /// An update was made and a relayout is required
    Relayout,
}




mod property_hash {
    use std::collections::HashMap;
    use crate::style::css::{CssProperty, StyleProperty};

    pub type PropertyHashMap<V=StyleProperty> = HashMap<CssProperty, V, FastU8Hash>;

    #[derive(Copy, Clone, Default)]
    pub struct FastU8Hash(u8);
    impl core::hash::Hasher for FastU8Hash {
        fn write(&mut self, bytes: &[u8]) {
            self.0 = bytes[0]
        }
        fn finish(&self) -> u64 {
            self.0 as u64
        }
    }

    impl core::hash::BuildHasher for FastU8Hash {
        type Hasher = Self;
        fn build_hasher(&self) -> Self::Hasher { Self(0) }
    }
    
}

