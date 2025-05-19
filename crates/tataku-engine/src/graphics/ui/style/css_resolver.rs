use crate::prelude::*;
use crate::prelude::ui::*;
use simplecss::StyleSheet;

const ROW_COL: &str = r#"
    row {
        flex-direction: row; 
    }
    column {
        flex-direction: column;
    }
"#;


struct Thingy<'a> {
    selector: simplecss::Selector<'a>,
    style: CssStyle,
}
impl<'a> Thingy<'a> {
    fn parse(rule: &simplecss::Rule<'a>) -> Self {
        Self {
            selector: rule.selector.clone(),
            style: CssStyle::parse_css(rule),
        }
    }
}

/// key is the % of the animation
/// ie "0%" or "100%" 
#[derive(Clone)]
pub struct CssAnimation(pub HashMap<u8, CssStyle>);
impl CssAnimation {
    pub fn new(body: &str) -> Self {
        let mut map = HashMap::new();

        let a = StyleSheet::parse(body);
        let parsed = a.rules
            .iter()
            .map(Thingy::parse)
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

    pub fn get(&self, val: u8) -> Option<&CssStyle> {
        self.0.get(&val)
    }
}

pub struct CssResolver<'a> {
    parsed: Vec<Thingy<'a>>,
    animations: HashMap<String, CssAnimation>,
}
impl<'a> CssResolver<'a> {
    pub fn new(
        style: &'a str,
    ) -> Self {
        let mut animations = HashMap::new();

        let mut style = StyleSheet::parse(style);
        style.parse_more(ROW_COL);
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
                        other => if let Ok(n) = other.parse::<u8>() { n } else { continue }
                    };
                    anim.insert(frame, style);
                }
                animations.insert(name.to_string(), CssAnimation(anim));
            }
        }
        
        let parsed = style.rules.iter()
            .map(Thingy::parse)
            .collect();

        Self {
            parsed,
            animations,
        }
    }

    // TODO: do we want to support !important?
    pub fn resolve_style(
        &mut self, 
        element_style: &str,
        node: NodeId,
        tree: &Tree,
    ) -> ElementStateStyles<()> {
        let a = format!("* {{ {element_style} }}");
        let e_stylesheet = StyleSheet::parse(&a);
        let e_style = e_stylesheet
            .rules
            .first()
            .map(CssStyle::parse_css)
            .unwrap_or_default();

        let mut states = ElementStateStyles::<()>::default();
        for (state, style) in [
            (ElementState::None, &mut states.none.0),
            (ElementState::Hover, &mut states.hover.0),
            (ElementState::Active, &mut states.active.0),
            (ElementState::Focus, &mut states.focus.0),
        ] {
            // resolve the element's style
            let mut ele_style = self
                .parsed
                .iter()
                .filter(|i| 
                    i.selector.matches(&fuck::A::new(tree, node, state))
                )
                .fold(
                    e_style.clone(), 
                    |a, b| a.merge(b.style.clone())
                );

            // resolve inheritance
            if let Some(parent) = tree.parent(node) {
                let ctx = tree.get_context(parent).unwrap();
                let parent_style = ctx
                    .element_data
                    .styles
                    .get_style(state); // FIXME: should this be ElementState::None??
                ele_style = ele_style.merge_parent(parent_style.0.clone());
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

    pub struct A<'a> {
        pub tree: &'a Tree,
        pub node: NodeId,
        state: ElementState
    }
    impl<'a> A <'a>{
        pub fn new(tree: &'a Tree, node: NodeId, state: ElementState) -> Self {
            Self {
                tree,
                node,
                state
            }
        }
        pub fn child_index(&self) -> Option<usize> {
            let parent = self.tree.parent(self.node)?;
            let children = self.tree.children(parent)?;
            let index = children
                .iter()
                .enumerate()
                .find(|(_, id)| id == &&self.node)
                ?.0;

            Some(index)
        }
    }

    impl simplecss::Element for A<'_> {
        fn parent_element(&self) -> Option<Self> {
            let parent = self.tree.parent(self.node)?;
            Some(Self::new(self.tree, parent, self.state))
        }
        
        fn prev_sibling_element(&self) -> Option<Self> {
            let parent = self.tree.parent(self.node)?;
            let children = self.tree.children(parent)?;

            let index = children
                .iter()
                .enumerate()
                .find(|(_, id)| id == &&self.node)
                ?.0;

            let sibling = *children.get(index - 1)?;

            Some(Self::new(self.tree, sibling, self.state))
        }
    
        fn has_local_name(&self, name: &str) -> bool {
            let Some(ctx) = self.tree.get_context(self.node) else {
                return false
            };

            ctx.element_data.element_name == name
        }
    
        fn attribute_matches(
            &self, 
            local_name: &str, 
            operator: simplecss::AttributeOperator<'_>
        ) -> bool {
            let Some(ctx) = self.tree.get_context(self.node) else { return false };
            
            match local_name {
                "id" => ctx.element_data.id.as_ref().map(|id| operator.matches(id)).unwrap_or_default(),
                "class" => operator.matches(&ctx.element_data.class_list.join(" ")),

                other => panic!("attribute_matches {other}")
            }
        }
    
        fn pseudo_class_matches(&self, class: simplecss::PseudoClass<'_>) -> bool {
            use simplecss::PseudoClass;
            if let PseudoClass::FirstChild = class {
                return self.child_index() == Some(0)
            }
            
            let state = self.state;
            match class {
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
        0% { display: grid; }
    }
    "#;
    let a = CssResolver::new(css);
    let anim = a.get_animation("test").expect("no anim?");

    let from = anim.get(0).unwrap();
    assert_eq!(from.display.value(), Some(&ui::DisplayType::Grid));

    let mid = anim.get(50).unwrap();
    assert_eq!(mid.display.value(), Some(&ui::DisplayType::None));

    let to = anim.get(100).unwrap();
    assert_eq!(to.display.value(), Some(&ui::DisplayType::Flex));
}
