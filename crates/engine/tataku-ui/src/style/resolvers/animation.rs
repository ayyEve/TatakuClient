use std::collections::HashMap;
use crate::style::css::StyleProperty;

/// key is the % of the animation
/// ie "0%" or "100%" 
#[derive(Clone)]
pub struct CssAnimation(pub HashMap<u8, Vec<StyleProperty>>);
impl CssAnimation {
    pub fn new(body: &str) -> Self {
        let mut map = HashMap::new();

        let a = simplecss::StyleSheet::parse(body);
        let parsed = a.rules
            .iter()
            .map(super::CssRuleStyleResolver::parse)
            .collect::<Vec<_>>();

        struct A<'a>(&'a str);
        impl simplecss::Element for A<'_> {
            fn has_local_name(&self, name: &str) -> bool { name == self.0 }

            // keyframe names dont have anything else
            fn parent_element(&self) -> Option<Self> { None }
            fn prev_sibling_element(&self) -> Option<Self> { None }
            fn attribute_matches(&self, _local_name: &str, _operator: simplecss::AttributeOperator<'_>) -> bool { false }
            fn pseudo_class_matches(&self, _class: simplecss::PseudoClass<'_>) -> bool { false }
        }

        for i in 0..=100 {
            let name = format!("{i}%");
            for p in &parsed {
                if p.selector.matches(&A(&name)) {
                    map.insert(i, p.style.clone());
                    break;
                }
            }
        }
        // try to parse `from` and `to`
        for p in &parsed {
            if p.selector.matches(&A("from")) {
                map.insert(0, p.style.clone());
            } else if p.selector.matches(&A("to")) {
                map.insert(100, p.style.clone());
            }
        }

        Self(map)
    }

    pub fn get(&self, val: u8) -> Option<&Vec<StyleProperty>> {
        self.0.get(&val)
    }
}
