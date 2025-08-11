
pub(super) struct CssRuleStyleResolver<'a> {
    pub selector: simplecss::Selector<'a>,
    pub style: crate::prelude::ui::CssStyle,
}
impl<'a> CssRuleStyleResolver<'a> {
    pub fn parse(rule: &simplecss::Rule<'a>) -> Self {
        Self {
            selector: rule.selector.clone(),
            style: crate::prelude::ui::CssStyle::parse_css(rule),
        }
    }
}
