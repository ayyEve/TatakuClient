use crate::*;
use crate::style::*;
use crate::tree::*;
use simplecss::StyleSheet;
use super::CssRuleStyleResolver;


const BASE_STYLE: &str = r#"
    row {
        flex-direction: row; 
    }
    column {
        flex-direction: column;
    }

    text {
        width: 100%;
    }
"#;

pub struct CssResolver<'a> {
    parsed: Vec<CssRuleStyleResolver<'a>>,
    animations: HashMap<String, CssAnimation>,
}
impl<'a> CssResolver<'a> {
    pub fn new(style_str: &'a str) -> Self {
        let mut animations = HashMap::new();

        let mut style = StyleSheet::parse(style_str);
        style.parse_more(BASE_STYLE);
        use simplecss::at_rules::at_rule::AtRule;
        for rule in style.at_rules.iter() {
            if let AtRule::Keyframes { name, frames } = rule {
                let mut anim = HashMap::new();
                for frame in frames {
                    let rule = simplecss::Rule {
                        selector: simplecss::Selector::parse("*").unwrap(),
                        declarations: frame.declarations.clone()
                    };
                    let style = CssStyle::parse_css(&rule);
                    let frame = match frame.key {
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
        node: &NodeId,
        tree: &Tree<Action>,
    ) -> ElementStateStyles<CssStyle, ()> {

        let a = format!("* {{ {element_style} }}");
        let base_stylesheet = StyleSheet::parse(&a);
        let base_style = base_stylesheet
            .rules
            .first()
            .map(CssStyle::parse_css)
            .unwrap_or_default();

        let mut states = ElementStateStyles::<CssStyle, ()>::default();
        for (state, style) in [
            (ElementState::None, &mut states.none.0),
            (ElementState::Hover, &mut states.hover.0),
            (ElementState::Active, &mut states.active.0),
            (ElementState::Focus, &mut states.focus.0),
        ] {
            // resolve the element's style
            let f = fuck::A::new(tree, *node, state);
            let mut ele_style = self
                .parsed
                .iter()
                .filter(|i| i.selector.matches(&f))
                .fold(
                    base_style.clone(), 
                    |a, b| a.merge(b.style.clone())                    
                );

            // resolve inheritance
            if let Some(parent) = tree.parent(node) {
                let ctx = tree.get_context(&parent).unwrap();
                let parent_style = ctx.get_style(state); // FIXME: should this be ElementState::None?
                ele_style = ele_style.merge_parent(parent_style.clone());
            }

            *style = ele_style;
        }

        states
    }

    pub fn get_animation(&self, name: &str) -> Option<CssAnimation> {
        self.animations.get(name).cloned()
    }
}

mod fuck {
    use super::*;

    pub struct A<'a, Action: Send + Sync> {
        pub tree: &'a Tree<Action>,
        pub node: NodeId,
        state: ElementState
    }
    impl<'a, Action: Send + Sync + 'static> A<'a, Action>{
        pub fn new(tree: &'a Tree<Action>, node: NodeId, state: ElementState) -> Self {
            Self {
                tree,
                node,
                state
            }
        }
        pub fn child_index(&self) -> Option<usize> {
            let parent = self.tree.parent(&self.node)?;
            let children = self.tree.children(&parent);
            children
                .iter()
                .position(|id| id == &self.node)
        }
    }

    impl<Action: Send + Sync + 'static> simplecss::Element for A<'_, Action> {
        fn parent_element(&self) -> Option<Self> {
            let parent = self.tree.parent(&self.node)?;
            Some(Self::new(self.tree, parent, self.state))
        }
        
        fn prev_sibling_element(&self) -> Option<Self> {
            let parent = self.tree.parent(&self.node)?;
            let children = self.tree.children(&parent);

            let index = children
                .iter()
                .position(|id| id == &self.node)?;

            let sibling = *children.get(index - 1)?;

            Some(Self::new(self.tree, sibling, self.state))
        }
    
        fn has_local_name(&self, name: &str) -> bool {
            let Some(ctx) = self.tree.get_context(&self.node) 
            else { return false };

            ctx.element_data.element_name == name
        }
    
        fn attribute_matches(
            &self, 
            local_name: &str, 
            operator: simplecss::AttributeOperator<'_>
        ) -> bool {
            let Some(ctx) = self.tree.get_context(&self.node) 
            else { return false };
            
            match local_name {
                "id" => ctx.element_data.id.as_ref().map(|id| operator.matches(id)).unwrap_or_default(),
                "class" => operator.matches(&ArcStr::join(&ctx.element_data.class_list, " ")),

                other => panic!("attribute_matches {other}")
            }
        }
    
        fn pseudo_class_matches(&self, class: simplecss::PseudoClass<'_>) -> bool {
            use simplecss::PseudoClass;
            let state = self.state;
            match class {
                PseudoClass::FirstChild => self.child_index() == Some(0),
                PseudoClass::Active => state.contains(ElementState::Active),
                PseudoClass::Hover => state.contains(ElementState::Hover),
                PseudoClass::Focus => state.contains(ElementState::Focus),

                _ => false,
            }
        }
    }
    
}

#[test]
fn test() {
    let css = r#"
    @keyframes test {
        100% { display: flex; }
        50% { display: none; }
        0% { display: block; }
    }
    "#;
    let a = CssResolver::new(css);
    let anim = a.get_animation("test").expect("no anim?");

    let from = anim.get(0).unwrap();
    assert_eq!(from.display.value(), Some(&DisplayType::Block));

    let mid = anim.get(50).unwrap();
    assert_eq!(mid.display.value(), Some(&DisplayType::None));

    let to = anim.get(100).unwrap();
    assert_eq!(to.display.value(), Some(&DisplayType::Flex));
}
