use tataku_engine_common::common::*;
use tataku_common::reflect::Reflect;
pub use property_hash::PropertyHashMap;
use crate::style::css::{
    CssStyle,
    CssProperty,
    StyleProperty,
};

#[derive(Clone, Default)]
pub struct StaticStyleLayer {
    pub properties: PropertyHashMap,
}
impl StaticStyleLayer {
    pub fn from_property_list(list: Vec<StyleProperty>) -> Self {
        Self {
            properties: list
                .into_iter()
                .map(|i| (i.css_property(), i))
                .collect(),
        }
    }

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
pub struct AnimatedStyleLayer {
    pub id: LayerId,
    pub enabled: bool,
    pub properties: PropertyHashMap,
}
impl AnimatedStyleLayer {
    pub fn from_property_list(
        list: Vec<StyleProperty>,
        id: LayerId,
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

    fn get_property(&self, property: CssProperty) -> Option<Cow<'_, StyleProperty>> {
        self.properties.get(&property).map(Cow::Borrowed)
    }
    fn set_property(&mut self, property: StyleProperty) {
        self.properties.insert(property.css_property(), property);
    }
    fn remove_property(&mut self, property: CssProperty) {
        self.properties.remove(&property);
    }


    fn update(&mut self, _time: f32, _values: &dyn Reflect) -> LayerUpdateResponse { 
        LayerUpdateResponse::None
    }
}


#[derive(Clone, Debug, Default)]
pub enum LayerId {
    #[default]
    Base,
    Overrides,

    Selector(simplecss::Selector<ArcStr>),
}
impl LayerId {
    pub fn is_selector(&self) -> bool {
        matches!(self, Self::Selector(_))
    }
}
impl From<simplecss::Selector<ArcStr>> for LayerId {
    fn from(value: simplecss::Selector<ArcStr>) -> Self {
        Self::Selector(value)
    }
}
impl PartialEq for LayerId {
    fn eq(&self, other: &LayerId) -> bool {
    
        match (self, other) {
            (Self::Base, Self::Base) => true,
            (Self::Overrides, Self::Overrides) => true,

            // (Self::Selector(e1), Self::Selector(e2)) => {
            //     // TODO: is this correct, or should we do some bitchecks?
            //     e1 == e2
            // }

            _ => false,
        }
    }
}


#[derive(Clone, Default)]
pub struct Style {
    cache: PropertyHashMap<Option<StyleProperty>>,

    layers: Vec<StyleLayer>,
}
impl Style {
    pub fn new(base: StyleLayer) -> Self {
        assert_eq!(base.id, LayerId::Base);

        Self {
            cache: PropertyHashMap::default(),
            layers: vec![ 
                base, 
                StyleLayer::new(LayerId::Overrides, StaticStyleLayer::default().into()) 
            ]
        }
    }

    pub fn menu_layout() -> Self {
        let list = CssStyle::menu_layout().into_property_list(true);
        let layer = StaticStyleLayer::from_property_list(list);

        Self::new(StyleLayer::new(LayerId::Base, layer.into()))
    }

    pub fn get(&self, property: CssProperty) -> Option<Cow<'_, StyleProperty>> {
        if let Some(cached) = self.cache.get(&property) {
            cached.as_ref().map(Cow::Borrowed)
        } else {
            self.layers
                .iter().rev()
                .filter(|i| i.enabled)
                .find_map(|i| i.get_property(property))
        }
    }
    pub fn get_resolved<T>(&self, property: CssProperty, values: &dyn Reflect) -> Option<T> 
        where T: Copy + Reflect + std::fmt::Debug {
        self
            .get(property)
            .and_then(|p| p.value::<T>()
            .resolve_copied(values))
    }

    pub fn get_layer(&mut self, id: &LayerId) -> Option<&mut StyleLayer> {
        self.layers
            .iter_mut()
            .find(|i| &i.id == id)
    }

    pub fn add_layer(&mut self, layer: StyleLayer) {
        self.layers.insert(self.layers.len() - 1, layer);
    }

    pub fn add_layer_unchecked(&mut self, layer: StyleLayer) {
        self.layers.push(layer);
    }
}


#[derive(Clone)]
pub struct StyleLayer {
    pub id: LayerId,
    pub enabled: bool,

    pub inner: StyleLayerType,
}
impl StyleLayer {
    pub fn empty_base() -> Self {
        Self { 
            id: LayerId::Base, 
            enabled: true, 
            inner: StyleLayerType::Static(Box::new(StaticStyleLayer::default()))
        }
    }

    pub fn new(id: LayerId, inner: StyleLayerType) -> Self {
        Self {
            id,
            enabled: true,
            inner,
        }
    }



    pub fn from_css_rule(id: Option<LayerId>, rule: &simplecss::Rule<ArcStr>) -> Self {
        let id = id.unwrap_or(LayerId::Selector(rule.selector.clone()));
        let inner = StyleLayerType::from_rule(rule);
        Self {
            enabled: !id.is_selector(),
            id,
            inner
        }
    }


    pub fn get_property(&self, property: CssProperty) -> Option<Cow<'_, StyleProperty>> {
        match &self.inner {
            StyleLayerType::Static(s) => s.get_property(property),
            StyleLayerType::Animated(a) => a.get_property(property),
        }
    }
    pub fn set_property(&mut self, property: StyleProperty) {
        match &mut self.inner {
            StyleLayerType::Static(s) => s.set_property(property),
            StyleLayerType::Animated(a) => a.set_property(property),
        }
    }
    pub fn remove_property(&mut self, property: CssProperty) {
        match &mut self.inner {
            StyleLayerType::Static(s) => s.remove_property(property),
            StyleLayerType::Animated(a) => a.remove_property(property),
        }
    }

    /// returns if anything was modified
    pub fn update(&mut self, time: f32, values: &dyn Reflect) -> LayerUpdateResponse {  
        match &mut self.inner {
            StyleLayerType::Static(_) => LayerUpdateResponse::None ,
            StyleLayerType::Animated(a) => a.update(time, values),
        }
    }

}


#[derive(Clone, From)]
pub enum StyleLayerType {
    Static(Box<StaticStyleLayer>),
    Animated(Box<AnimatedStyleLayer>),
}
impl StyleLayerType {
    pub fn from_rule(rule: &simplecss::Rule<ArcStr>) -> Self {
        let list = CssStyle::parse_css(rule)
            .into_property_list(true);

        Self::Static(Box::new(StaticStyleLayer::from_property_list(list)))
    }
}

impl From<StaticStyleLayer> for StyleLayerType {
    fn from(value: StaticStyleLayer) -> Self {
        Self::Static(Box::new(value))
    }
}



#[derive(Copy, Clone, Debug, Default)]
pub enum LayerUpdateResponse {
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
            self.0 = bytes[0];
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
