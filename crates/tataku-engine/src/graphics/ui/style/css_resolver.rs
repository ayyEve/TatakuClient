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

pub struct CssResolver<'a> {
    _style: StyleSheet<'a>,
    parsed: Vec<Thingy<'a>>,
}
impl<'a> CssResolver<'a> {
    pub fn new(
        style: &'a str,
    ) -> Self {
        let mut style = StyleSheet::parse(style);
        style.parse_more(ROW_COL);
        
        let parsed = style.rules.iter()
            .map(Thingy::parse)
            .collect();

        Self {
            _style: style,
            parsed,
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
        let e_style = e_stylesheet.rules
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
            let mut ele_style = self.parsed.iter()
                .filter(|i| i.selector.matches(&fuck::A::new(tree, node, state)))
                .fold(e_style.clone(), |a, b| a.merge(b.style.clone()));

            // resolve inheritance
            if let Some(parent) = tree.parent(node) {
                let ctx = tree.get_context(parent).unwrap();
                let parent_style = ctx.element_data.styles.get_style(ElementState::None);
                ele_style = ele_style.merge_parent(parent_style.0.clone());
            }

            *style = ele_style;
        }

        states
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

            // let Some(ctx) = self.tree.get_context(self.node) else {
            //     return false
            // };

            // let state = &ctx.element_data.state;
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
