use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct BuildableCalc(Arc<Vec<ShuntingYardToken>>);
impl BuildableCalc {
    pub fn parse(expr: impl AsRef<str>) -> ShuntingYardResult<Self> {
        let expr = expr.as_ref();
        let tokens = ShuntingYard::parse_expression(expr)?;
        Ok(Self(Arc::new(tokens)))
    }

    pub fn resolve<'a>(&self, values: &'a dyn Reflect) -> ShuntingYardResult<Cow<'a, TatakuValue>> {
        ShuntingYard::evaluate_rpn(&self.0, values)
    }
}
