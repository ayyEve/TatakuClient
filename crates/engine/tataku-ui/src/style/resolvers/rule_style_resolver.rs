use crate::style::css::{CssStyle, StyleProperty};

pub(super) struct CssRuleStyleResolver<'a> {
    pub selector: simplecss::Selector<'a>,
    pub style: Vec<StyleProperty>,
}
impl<'a> CssRuleStyleResolver<'a> {
    pub fn parse(rule: &simplecss::Rule<'a>) -> Self {
        Self {
            selector: rule.selector.clone(),
            style: CssStyle::parse_css(rule).into_property_list(true),
        }
    }
}
