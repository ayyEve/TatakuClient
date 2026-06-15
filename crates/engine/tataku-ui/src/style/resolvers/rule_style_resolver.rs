use tataku_engine_common::data::ArcStr;

use crate::style::css::{CssStyle, StyleProperty};

pub(super) struct CssRuleStyleResolver {
    pub selector: simplecss::Selector<ArcStr>,
    pub style: Vec<StyleProperty>,
}
impl CssRuleStyleResolver {
    pub fn parse(rule: &simplecss::Rule<ArcStr>) -> Self {
        Self {
            selector: rule.selector.clone(),
            style: CssStyle::parse_css(rule).into_property_list(true),
        }
    }
}
