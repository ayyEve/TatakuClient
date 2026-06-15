use crate::style::css::CssStyle;
use crate::*;
use crate::tree::*;
use crate::style::*;
use simplecss::StyleSheet;
use super::CssRuleStyleResolver;

pub struct CssResolver {
    parsed: Vec<CssRuleStyleResolver>,
    animations: HashMap<String, CssAnimation>,
}
impl CssResolver {
    pub fn new(
        style_str: &str, 
        default_css: &str,
    ) -> Self {
        let mut animations = HashMap::new();

        let mut style = StyleSheet::parse(default_css);
        style.parse_more(style_str);

        use simplecss::at_rules::at_rule::AtRule;
        for rule in style.at_rules.iter() {
            if let AtRule::Keyframes { name, frames } = rule {
                let mut anim = HashMap::new();
                for frame in frames {
                    let rule = simplecss::Rule {
                        selector: simplecss::Selector::parse("*").unwrap(),
                        declarations: frame.declarations.clone()
                    };
                    let style = CssStyle::parse_css(&rule).into_property_list(true);
                    let frame = match &*frame.key {
                        "from" => 0,
                        "to" => 100,
                        other => if let Ok(n) = other.parse::<u8>() { n } 
                            else { continue }
                    };
                    anim.insert(frame, style);
                }
                animations.insert((*name).to_string(), CssAnimation(anim));
            }
        }
        
        let parsed = style.rules.iter()
            .map(CssRuleStyleResolver::parse)
            .collect::<Vec<_>>();

        Self {
            parsed,
            animations,
        }
    }

    // TODO: do we want to support !important?
    pub fn resolve_style<Action: Send + Sync + 'static>(
        &mut self, 
        element_style: &str,
        node: NodeId,
        tree: &Tree<Action>,
    ) -> Style {
        let a = format!("* {{ {element_style} }}");
        let base_stylesheet = StyleSheet::<ArcStr>::parse(&a);
        let base_layer = base_stylesheet.rules
            .first()
            .map_or(
                StyleLayer::empty_base(), 
                |r| StyleLayer::from_css_rule(Some(LayerId::Base), r)
            );

        let mut style = Style::new(base_layer);
        
        let f = fuck::CanMatch::new(tree, node);
        for i in self.parsed.iter() {
            use fuck::MatchResult;
            match f.matches(&i.selector) {
                MatchResult::NoMatch => {},

                // TODO: optimize when things will always match, ie combine them into a single layer
                // just need to consider specificity order when implementing that 
                MatchResult::WillAlwaysMatch 
                | MatchResult::Matches => {
                    let layer = StyleLayer::new(
                        LayerId::Selector(i.selector.clone()), 
                        StaticStyleLayer::from_property_list(i.style.clone()).into()
                    );
                    style.add_layer(layer);
                }
            }
        }

        style
    }

    pub fn get_animation(&self, name: &str) -> Option<CssAnimation> {
        self.animations.get(name).cloned()
    }
}

mod fuck {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    pub enum MatchResult {
        NoMatch,
        Matches,
        WillAlwaysMatch,
    }

    pub struct CanMatch<'a, Action: Send + Sync> {
        pub tree: &'a Tree<Action>,
        pub node: NodeId,

        has_pseudo_class: std::cell::Cell<bool>,
    }
    impl<'a, Action: Send + Sync + 'static> CanMatch<'a, Action>{
        pub fn new(tree: &'a Tree<Action>, node: NodeId) -> Self {
            Self {
                tree,
                node,
                has_pseudo_class: std::cell::Cell::new(false),
            }
        }
        pub fn child_index(&self) -> Option<usize> {
            let parent = self.tree.parent(self.node)?;
            let children = self.tree.children(parent);
            children
                .iter()
                .position(|id| id == &self.node)
        }

        pub fn matches(&self, selector: &simplecss::Selector<ArcStr>) -> MatchResult {
            self.has_pseudo_class.set(false);

            let matches = selector.matches(self);

            match (matches, self.has_pseudo_class.get()) {
                (false, _) => MatchResult::NoMatch,
                (true, true) => MatchResult::Matches,
                (true, false) => MatchResult::WillAlwaysMatch,
            }
        }
    }

    impl<Action: Send + Sync + 'static> simplecss::Element<ArcStr> for CanMatch<'_, Action> {
        fn parent_element(&self) -> Option<Self> {
            let parent = self.tree.parent(self.node)?;
            Some(Self::new(self.tree, parent))
        }
        
        fn prev_sibling_element(&self) -> Option<Self> {
            let parent = self.tree.parent(self.node)?;
            let children = self.tree.children(parent);

            let index = children
                .iter()
                .position(|id| id == &self.node)?;

            let sibling = *children.get(index - 1)?;

            Some(Self::new(self.tree, sibling))
        }
    
        fn has_local_name(&self, name: &str) -> bool {
            let Some(ctx) = self.tree.get_context(self.node) 
            else { return false };

            ctx.element_data.element_name == name
        }
    
        fn attribute_matches(
            &self, 
            local_name: &str, 
            operator: &simplecss::AttributeOperator<ArcStr>
        ) -> bool {
            let Some(ctx) = self.tree.get_context(self.node) 
            else { return false };
            
            match local_name {
                "id" => ctx.element_data.id.as_ref().map(|id| operator.matches(id)).unwrap_or_default(),
                "class" => operator.matches(&ArcStr::join(&ctx.element_data.class_list, " ")),

                other => panic!("attribute_matches {other}")
            }
        }
    
        // always return true since pseudoclasses can change
        fn pseudo_class_matches(&self, class: &simplecss::PseudoClass<ArcStr>) -> bool {
            if let simplecss::PseudoClass::FirstChild = class {
                return self.child_index() == Some(0)
            }


            self.has_pseudo_class.set(true);
            true
            // use simplecss::PseudoClass;
            // let state = self.state;
            // match class {
            //     PseudoClass::FirstChild => self.child_index() == Some(0),
            //     PseudoClass::Active => state.contains(ElementState::Active),
            //     PseudoClass::Hover => state.contains(ElementState::Hover),
            //     PseudoClass::Focus => state.contains(ElementState::Focus),

            //     _ => false,
            // }
        }
    }
    
}

// #[test]
// fn test() {
//     let css = r#"
//     @keyframes test {
//         100% { display: flex; }
//         50% { display: none; }
//         0% { display: block; }
//     }
//     "#;
//     let a = CssResolver::new(css, "");
//     let anim = a.get_animation("test").expect("no anim?");

//     let from: &CssPropertyCollection = anim.get(0).unwrap();
//     let mut style = CssStyle::default();

//     style.merge_with_collection(from);
//     assert_eq!(style.display.value(), Some(&DisplayType::Block));

//     let mid = anim.get(50).unwrap();
//     style.merge_with_collection(mid);
//     assert_eq!(style.display.value(), Some(&DisplayType::None));

//     let to = anim.get(100).unwrap();
//     style.merge_with_collection(to);
//     assert_eq!(style.display.value(), Some(&DisplayType::Flex));
// }
