use crate::prelude::*;
use tataku::TatakuValue;
use common::reflect::Reflect;
use tataku::GenericShuntingYard;
use engine::data::shunting_yards::buildable::{
    ShuntingYardResult,
    BuildableShuntingYard,
    BuildableShuntingYardToken,
};

#[derive(Clone, Debug, PartialEq)]
pub struct BuildableCalc(Arc<Vec<BuildableShuntingYardToken>>);

#[cfg(feature="graphics")]
impl BuildableCalc {
    pub fn parse(expr: impl AsRef<str>) -> ShuntingYardResult<Self> {
        let expr = expr.as_ref();
        let tokens = BuildableShuntingYard::parse_expression(expr)?;
        Ok(Self(Arc::new(tokens)))
    }

    pub fn resolve<'a>(
        &self, 
        values: &'a dyn Reflect
    ) -> ShuntingYardResult<Cow<'a, TatakuValue>> {
        BuildableShuntingYard::evaluate_rpn(&self.0, values)
    }
}
