use crate::prelude::*;

#[derive(Clone)]
pub enum CustomElementVariable {
    Raw(String),
    Variable(String),
    Calc(CustomElementCalc),
}

#[derive(Clone, Debug)]
pub struct CustomElementCalc(Vec<ShuntingYardToken>);
impl CustomElementCalc {
    pub fn parse(expr: impl AsRef<str>) -> ShuntingYardResult<Self> {
        let expr = expr.as_ref();
        let tokens = ShuntingYard::parse_expression(expr)?;
        Ok(Self(tokens))
    }

    pub fn resolve(&self, values: &ValueCollection) -> ShuntingYardResult<SYStackValue> {
        ShuntingYard::evaluate_rpn(&self.0, values)
    }
}
