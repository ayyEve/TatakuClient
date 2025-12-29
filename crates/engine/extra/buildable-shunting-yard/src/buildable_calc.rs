use crate::*;
use crate::common::reflect::Reflect;
use ::shunting_yard::ShuntingYard;

#[derive(Clone, Debug, PartialEq)]
pub struct BuildableCalc {
    tokens: Arc<Vec<crate::Token>>,
    pub expr: ArcStr,
}
impl BuildableCalc {
    pub fn parse(expr: ArcStr) -> ShuntingYardResult<Self> {
        let tokens = BuildableShuntingYard::parse_expression(&expr)?;
        Ok(Self {
            tokens: Arc::new(tokens),
            expr
        })
    }

    pub fn resolve<'a>(
        &self, 
        values: &'a dyn Reflect
    ) -> ShuntingYardResult<Cow<'a, tataku::TatakuValue>> {
        BuildableShuntingYard::evaluate_rpn(&self.tokens, values)
    }
}

