use crate::prelude::*;
use super::*;

type AstResult = Result<Vec<PathShuntingYardToken>, PathShuntingYardError>;

/// resolves variable paths, ie tmp.some_map::_something.id::game.blah
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(from="String", into="String")]
pub struct VariablePathResolver {
    pub(crate) var: ArcStr,
    ast: Arc<AstResult>,
}
impl VariablePathResolver {
    pub fn new(path: impl Into<ArcStr>) -> Self {
        let path = path.into();
        let ast = Arc::new(
            PathShuntingYard::parse_expression(&path)
        );
        
        Self {
            var: path,
            ast,
        }
    }

    pub fn resolve_path(&self, values: &dyn Reflect) -> TatakuResult<String> {
        match &*self.ast {
            Ok(rpn) => {
                let p = PathShuntingYard::evaluate_rpn(
                    rpn, 
                    values
                )
                .map_err(|e| TatakuError::String(format!("{e:?}")))?;
                
                Ok(p)
            }
            Err(e) => Err(TatakuError::String(format!("{e:?}"))),
        }
    }
}
impl From<String> for VariablePathResolver {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&str> for VariablePathResolver {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}
impl PartialEq for VariablePathResolver {
    fn eq(&self, other: &Self) -> bool {
        self.var.eq(&other.var)
    }
}
impl From<VariablePathResolver> for String {
    fn from(value: VariablePathResolver) -> Self {
        value.var.to_string()
    }
}

impl Display for VariablePathResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.var.fmt(f)
    }
}
