use crate::prelude::CssStyle;

pub(super) struct CssRuleStyleResolver<'a> {
    pub selector: simplecss::Selector<'a>,
    pub style: CssStyle,
}
impl<'a> CssRuleStyleResolver<'a> {
    pub fn parse(rule: &simplecss::Rule<'a>) -> Self {
        Self {
            selector: rule.selector.clone(),
            style: CssStyle::parse_css(rule),
        }
    }
}
